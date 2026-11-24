from worlds.LauncherComponents import Component, Type, components, launch


def run_client(*args: str) -> None:
    from .client import launch as launch_client
    launch(launch_client, name="OOTRR Client", args=args)


components.append(Component("OOTRR Client", func=run_client, game_name="OOTRR", component_type=Type.CLIENT, supports_uri=True))
