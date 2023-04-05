# Design

To start off on the right foot, its important to understand the basic design of the framework. Starting with the thing
that players will interact the most with, the units and map.

#### The Map and Things on the Map

In Bevy_GGF, there are two conceptual parts that comprise everything to do with the map. There is the actual map, built
using [Bevy_Ecs_Tilemap](https://github.com/StarArawn/bevy_ecs_tilemap), and then there is everything that goes on top
of the map, what we call Objects. Objects are everything that is not a literal map tile - units, buildings, boxes,
whatever your game needs. For all of our examples, we will use a fictional Advance Wars style game and occasionally a 4x
style.

These are the minimum components needed for an object to exist on the map.

```rust
#[derive(Bundle)]
pub struct ObjectMinimalBundle {
    pub object: Object,
    pub object_info: ObjectInfo,
    pub object_grid_position: ObjectGridPosition,
    pub object_stacking_class: ObjectStackingClass,
    pub sprite_bundle: SpriteBundle,
}
```

To briefly explain what these components are, in order:

- __Object__ is a marker component denoting this entity as a well... object.
- __ObjectInfo__ holds references to the type, group, and class of the object. This will be explained more in depth
  later
  but you can think of this as representing what an object is.
    - __ObjectClass__ is the broadest category of objects. This could be something like _Air_, _Ground_, _Building_ and
      _Sea_.
    - __ObjectGroup__ is still broad but more specific. It could be _Armored_, _Infantry_, _CapitalShip_, _FixedWing_,
      _Rotary_,
      _Improvement_, _Production_, _Fortification_.
    - __ObjectType__ is the most specific representing one specific type of object. _LightInfantry_, _MediumTank_,
      _UtilityHelicopter_, _Bridge_.

  Together these three categories are combined to create layers of functionality and inheritance. A LightInfantry type
  unit has an ObjectGroup of Infantry and an ObjectClass of Ground. Meaning any buffs, abilites, etc that apply to any
  of the three apply to this specific unit. The same could be said for something like Destroyer > Escort > Water.
- ObjectGridPosition is just that. The grid position of the object. Currently it just holds a Bevy_Ecs_Tilemap
  TilePosition struct however this is potentially going to be updated with stuff like a chunk position or whatever else
  is needed.
- ObjectStackingClass is used to denote what stacks the object uses in a tile. If you want all the units in the game to
  share the same space on a tile then you just make one StackingClass. However if you want say, air units to be able to
  fly above ground units, then you should make an air unit stacking class and a ground unit stacking class. The movement
  system will keep track of whats in what.
- SpriteBundle is the built in Bevy Sprite Bundle. Needed if you want anything to show up, however the Transform
  component is used in internal systems like the movement system which means ghost units aren't allowed.

Those are the current core object components. The secret sauce of the entire framework comes through the dynamic
combination of the optional components. See the wiki for a list of all the optional
components [here](https://github.com/NoahShomette/bevy_ggf/wiki/Object-Components)

Essentially though, if you want an object to be able to move, simply add the movement components, If you want an object
to be able to attack, add the attack components. If you want an object to generate a certain amount of resources at the
beginning or end of a turn, add that component. And on and on. By utilizing the strengths of the ECS system we can
create an incredibly robust and dynamic system.

An example - to make a building like in Advance Wars.
Add the basic object components, add a supplier component, add a generate gold on turn begin component, add a production
component with the list of objects that you want it to be able to produce, add a selectable component, and there you go!
A building that supplies objects around it, can build units, and can generate resources. And because you didnt add any
movement or combat components than the building just can't do either of those things. However if you wanted to add them
then you could and that would be valid, the systems don't care.

You are free to make your own components as well, theres nothing inherently special about an object, it's just an
Entity. Make your own components, add them, do your thing!