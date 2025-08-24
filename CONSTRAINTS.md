# Ant Colony Simulation Challenge Rules

## Purpose
These constraints prevent optimization shortcuts that would make the simulation less realistic or educational. The goal is to improve ant-like behavior through better pheromone algorithms and movement logic, not by making the challenge easier.

## Core Principle
**Optimize the ants, not the world.** Changes should focus on making individual ant behavior more sophisticated, realistic, and effective within realistic biological constraints.

## Mandatory Constraints

### World Design Rules
- **DO NOT** reduce minimum food distance from nest (currently 333+ units)
- **DO NOT** increase food source size or detection radius beyond realistic proportions
- **DO NOT** reduce food quantity or regeneration requirements below realistic levels
- **DO NOT** make the world smaller to artificially reduce travel distances
- **DO NOT** add unrealistic "teleportation" or instant movement mechanics

### Ant Perception Rules
- **DO NOT** allow ants to see the entire world state
- **DO NOT** give ants perfect knowledge of food or nest locations beyond their sensing radius
- **DO NOT** allow ants to directly communicate complex information (positions, directions) without pheromone intermediation
- **DO NOT** implement unrealistic sensory ranges (ants should have limited local perception)

### Pheromone System Rules
- **DO NOT** make pheromones permanent or remove natural decay
- **DO NOT** allow pheromones to carry complex data structures (they should be simple chemical signals)
- **DO NOT** make pheromone detection ranges unrealistically large
- **DO NOT** remove the need for ants to physically traverse and maintain trails

### Movement and Physics Rules  
- **DO NOT** allow ants to move through walls or obstacles (when added)
- **DO NOT** implement unrealistic speeds or acceleration
- **DO NOT** remove collision detection or spatial constraints
- **DO NOT** allow ants to carry unlimited food or move while overloaded

### Performance Metric Rules
- **DO NOT** optimize metrics by reducing challenge difficulty
- **DO NOT** change success criteria to make poor performance appear good
- **DO NOT** ignore failing ants in aggregate performance calculations
- **DO NOT** artificially boost metrics through parameter manipulation rather than behavior improvement

## Encouraged Improvements

### Ant Behavior Enhancements
- **DO** improve pheromone following algorithms (trail quality assessment, gradient following)
- **DO** enhance stuck detection and recovery mechanisms  
- **DO** implement better exploration strategies (random walk improvements, momentum)
- **DO** add realistic ant memory and learning capabilities
- **DO** improve state transition logic (exploring → collecting → carrying)

### Pheromone System Improvements
- **DO** implement more sophisticated pheromone types (food, nest, warning, exploration)
- **DO** add realistic pheromone intensity and decay modeling
- **DO** improve pheromone trail reinforcement mechanisms
- **DO** implement pheromone trail optimization (removing redundant paths)

### Emergent Behavior Goals
- **DO** encourage efficient collective foraging patterns
- **DO** promote effective trail establishment and maintenance
- **DO** support adaptive responses to changing conditions
- **DO** enable cooperative problem-solving without direct communication

## Measurement Guidelines

### Valid Performance Improvements
- Increased successful food deliveries through better pathfinding
- Reduced average return times through improved trail following
- Lower "time since last goal achieved" across all ants
- More ants actively participating (fewer stuck/lost ants)
- Emergent trail optimization and redundancy elimination

### Invalid "Improvements"
- Higher metrics due to reduced challenge difficulty
- Better performance through unrealistic ant capabilities  
- Success through violation of biological plausibility
- Optimization that ignores individual ant effectiveness

## Current Challenge Level: "Distant Food Sources"
- Minimum food distance: 333 units from nest
- Maximum food distance: 500 units from nest  
- World size: 1000x1000 units
- No obstacles or hazards (yet)
- Single food type with standard nutritional value

## Future Challenge Progression
Each new challenge level should maintain all previous constraints while adding new realistic difficulties:
1. **Obstacles**: Rocks/walls requiring pathfinding around barriers
2. **Hazards**: Dangerous areas that require avoidance learning
3. **Multiple Colonies**: Competition for resources
4. **Dynamic Environment**: Moving obstacles, seasonal changes
5. **Specialized Roles**: Scout/worker/soldier differentiation

## Violation Consequences
Breaking these rules defeats the educational purpose. If constraints are violated:
1. Revert the change that violated the constraint
2. Implement the improvement within the constraint boundaries  
3. Focus on biological plausibility over metric optimization

## Review Process
Before implementing major changes, verify:
- [ ] Does this make ants more realistic and sophisticated?
- [ ] Does this maintain the challenge difficulty?
- [ ] Would real ants be capable of this behavior?
- [ ] Does this improve collective intelligence through individual improvements?

The goal is realistic ant behavior that emerges from simple rules, not artificial intelligence that happens to look like ants.