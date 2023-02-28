# Combat

We dont want something to be truly monolithic because we want parts replaceable
CORE: Battle Resolver - handles the actual battles. Give it entities, it processes the battle and spits out the result.

ANCILLARY PIECES:
- Range
- Fog of war
- move then attack
- Ammo
- Buffs/Nerfs - This is the only one thats probably integral to the Battle Resolver. Since this is important to accurately resolving battles.
- Optional method for special attacks


Build Battle Resolver than add extra bits. Battle Resolver design?

Pass two entities in, it gets the necessary components, runs the calculation, and then emits the result.
Keep Bat
