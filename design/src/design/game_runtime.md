# Goals

- Provide a game runtime that handles players, turns, game ticks, player actions, extra phases, and more.
- The runtime should provide systems that can handle keeping track of players in the game, updating them when its their
  turn, running registered systems at the beginning, during, and end of players turns, etc.
-

# Ideas:
- use a new custom schedule. Idk if this allows runtime editing of custom schedules.