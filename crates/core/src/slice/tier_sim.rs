#[cfg(test)]
mod tests {
    use std::collections::BTreeSet as BS;
    use std::path::PathBuf as PB;

    use crate::bfevfl::Flow;
    use crate::error::Res;
    use crate::flow_sim::db::Db;
    use crate::flow_sim::load::{Disk, Src};
    use crate::flow_sim::run::{End, Env, Fin, Mach, Opts, Sim};
    use crate::flow_sim::state::State;
    use crate::progressive::{entry_name, owners, tiers, wrap, FAMILIES};
    use crate::romfs::Romfs;
    use crate::tier_chain::{of, resolve};

    const CHESTS: [&str; 2] = ["Dm_SY_TreasureboxSmall", "Dm_SY_Treasurebox"];

    struct Wrapped {
        disk: Disk,
    }

    impl Src for Wrapped {
        fn flow(&mut self, name: &str) -> Res<Option<Flow>> {
            let f: Option<Flow> = self.disk.flow(name)?;
            match f {
                Some(mut x) if CHESTS.contains(&name) => {
                    for (fam, _) in FAMILIES {
                        wrap(&mut x, fam)?;
                    }
                    Ok(Some(x))
                }
                x => Ok(x),
            }
        }

        fn event(&mut self, name: &str) -> Res<Option<(String, String)>> {
            self.disk.event(name)
        }

        fn cuts(&mut self, name: &str) -> Vec<String> {
            self.disk.cuts(name)
        }
    }

    fn root() -> Option<PB> {
        let p: PB = std::env::var_os("OOTRR_ROMFS").map(PB::from).unwrap_or_else(|| PB::from("C:/o/nup dump/updates/131072/romfs"));
        p.join("Pack").join("ZsDic.pack.zs").is_file().then_some(p)
    }

    fn sim(p: &PB) -> Sim {
        let mut rf: Romfs = Romfs::open(p).unwrap();
        let db: Db = Db::load(&mut rf).unwrap();
        let src: Wrapped = Wrapped { disk: Disk::new(Romfs::open(p).unwrap(), None, Vec::new()) };
        Sim::new(db, Box::new(src), Opts::default())
    }

    fn gives(s: &mut Sim, flow: &str, fam: &str, st: State) -> (BS<String>, Vec<String>) {
        let t: &[&str] = tiers(fam).unwrap();
        let m: Mach = s.start(flow, &entry_name(fam), st, Env::new()).unwrap();
        let fins: Vec<Fin> = s.explore(m);
        let mut got: BS<String> = BS::new();
        let mut ends: Vec<String> = Vec::new();
        for f in &fins {
            if f.end != End::Done {
                ends.push(format!("{:?}", f.end));
            }
            for k in f.m.gr.keys() {
                if t.contains(&k.as_str()) {
                    got.insert(k.clone());
                }
            }
        }
        (got, ends)
    }

    #[test]
    #[ignore]
    fn the_real_chest_flows_follow_the_chain_for_every_owned_combination() {
        let Some(p) = root() else {
            eprintln!("skipped: no romfs, set OOTRR_ROMFS");
            return;
        };
        let mut s: Sim = sim(&p);
        let mut bad: Vec<String> = Vec::new();
        let mut runs: usize = 0;
        for flow in CHESTS {
            for (fam, _) in FAMILIES {
                let t: &[&str] = tiers(fam).unwrap();
                let o: &[&str] = owners(fam).unwrap();
                let n: usize = t.len();
                for mask in 0..1usize << n {
                    let owned: Vec<bool> = (0..n).map(|i| mask >> i & 1 == 1).collect();
                    let mut st: State = State::new();
                    s.settle(&mut st);
                    for (i, x) in o.iter().enumerate() {
                        if owned[i] {
                            s.own(&mut st, x, 1);
                        }
                    }
                    let want: usize = resolve(&of(fam, n), &owned).unwrap();
                    let (got, ends): (BS<String>, Vec<String>) = gives(&mut s, flow, fam, st);
                    runs += 1;
                    let line: String = format!("{flow}\t{fam}\t{owned:?}\twant {}\tgot {got:?}\tends {ends:?}", t[want]);
                    println!("SIM\t{line}");
                    if got.len() != 1 || !got.contains(t[want]) {
                        bad.push(line);
                    }
                }
            }
        }
        println!("RUNS {runs} BAD {}", bad.len());
        for b in &bad {
            println!("DIFF\t{b}");
        }
        for (k, v) in &s.assumed {
            println!("NOTE\t{k} x{v}");
        }
        for u in &s.unknown {
            println!("UNKNOWN\t{u}");
        }
        assert!(bad.is_empty());
    }

    #[test]
    #[ignore]
    fn three_chests_in_a_row_hand_out_the_tiers_in_order_when_grants_credit_the_owner() {
        let Some(p) = root() else {
            eprintln!("skipped: no romfs, set OOTRR_ROMFS");
            return;
        };
        let mut s: Sim = sim(&p);
        for (fam, _) in FAMILIES {
            let t: &[&str] = tiers(fam).unwrap();
            let o: &[&str] = owners(fam).unwrap();
            let mut st: State = State::new();
            s.settle(&mut st);
            let mut seq: Vec<String> = Vec::new();
            for _ in 0..t.len() + 1 {
                let (got, _): (BS<String>, Vec<String>) = gives(&mut s, CHESTS[0], fam, st.clone());
                let Some(g) = got.iter().next().cloned() else {
                    seq.push("nothing".to_string());
                    break;
                };
                let i: usize = t.iter().position(|x| *x == g).unwrap();
                s.own(&mut st, o[i], 1);
                seq.push(g);
            }
            println!("SEQ\t{fam}\t{}", seq.join(" then "));
            assert_eq!(seq[..t.len()].iter().map(|x| x.as_str()).collect::<Vec<&str>>(), t.to_vec(), "{fam}");
        }
    }
}
