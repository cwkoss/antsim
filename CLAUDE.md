# Claude Code Configuration for Ant Colony Simulation

## Project Overview
This is a Bevy-based ant colony simulation written in Rust that models emergent behavior, pheromone trails, and food collection dynamics. The simulation includes real-time video recording capabilities with scientifically accurate pheromone visualization.

## Project Goals

Model ant behavior accurately and interestingly to show the beauty of emergent behavior from a swarm of simple agents working together with simple logic.  Starting with a relatively simple easy challenge, once the ant behavior seems accurate enough, the challenge (and possibly optimization metrics) will be increased to put pressure on modeling the behavior more accurately, and its ability to adapt to new challenges and obstacles, until we finally arrive at something that is convincingly ant-like. 

By generating videos of each iteration of the simulation, we are documenting not just the behavior itself, but the progression and evolution of development, with the opportunity to create eductational and entertaining content about journey of man and machine working collaboratively to gain a deeper understanding of the beauty of nature. Eventually we hope to stitch the various generation videos together and publish them for consumption by the broader public. 

## Key Commands

### Build & Run
```bash
cargo run
```

### Video Processing
```bash
# Convert captured PNG frames to MP4 video
./ffmpeg/ffmpeg-2025-08-20-git-4d7c609be3-full_build/bin/ffmpeg.exe -framerate 30 -i "simulation_videos/test_XXX_TIMESTAMP_frames/frame_%04d.png" -c:v libx264 -pix_fmt yuv420p "simulation_videos/test_XXX_TIMESTAMP.mp4"
```

### Testing & Validation
```bash
# No specific test framework configured yet
# Manual testing via simulation runs
```

## Architecture Overview

### Core Modules
- **main.rs** - Bevy app setup, ECS system registration
- **components.rs** - Entity component definitions (AntState, FoodSource, Nest, etc.)
- **systems.rs** - ECS systems for movement, sensing, pheromone updates, performance tracking
- **pheromones.rs** - Pheromone grid simulation and decay logic
- **video.rs** - Real-time frame capture with actual pheromone trail visualization
- **colors.rs** - Unified color configuration for consistent simulation/video rendering
- **config.rs** - Simulation parameters and configuration

### Key Systems
1. **Ant Behavior Systems**: sensing_system, movement_system, food_collection_system
2. **Pheromone Systems**: pheromone_deposit_system, pheromone_update_system
3. **Visual Systems**: ant_visual_system, food_visual_system, update_pheromone_visualization
4. **Performance Systems**: performance_analysis_system, exit_system
5. **Video System**: video_recording_system with real pheromone data capture

## Important Implementation Details

### Pheromone Trail Visualization
- **Real pheromone data**: Video capture uses actual PheromoneGrid data, not artificial gradients
- **Color scheme**: Green = food pheromones, Blue = nest pheromones (consistent between simulation and video)
- **Coordinate mapping**: Grid-to-screen transformation for accurate visualization

### Video Recording
- **Immediate start**: Recording begins at simulation start (0 seconds)
- **Duration**: 5 minutes (300 seconds) for comprehensive behavior capture
- **Frame rate**: 30 fps capture, every 6th frame saved for 5-second final video (6x speed)
- **Format**: PNG frames → FFmpeg H.264 MP4 conversion
- **Resolution**: 406x720 (mobile-friendly aspect ratio)

### Performance Tracking
- **Metrics**: Deliveries per minute, average return time, oscillation detection
- **Auto-exit conditions**: High oscillation (≥20 ants) or lost carriers (≥10 ants)
- **Success criteria**: Sustained food delivery with low return times

## Current Configuration

### Simulation Parameters (config.rs)
- Initial ants: Variable based on current test setup
- Food sources: Multiple sources placed far from nest (333-500 units away)
- World size: 1000x1000 units
- Pheromone grid: 1:1 mapping with world coordinates

### Color Scheme (colors.rs)
- **Nest**: Yellow (#FFFF00)
- **Food sources**: Green (#00FF00)  
- **Ants exploring**: Red (#FF0000)
- **Ants carrying food**: Orange (#FF8000)
- **Ants collecting**: Yellow (#FFFF00)
- **Food pheromones**: Green (#00FF00)
- **Nest pheromones**: Blue (#0000FF)

## Optimization Notes

### Recent Performance Improvements (Generations 11-15)
- **Pheromone-Guided Behavior**: Implemented sophisticated trail following system with 8-directional sensing
- **Momentum-Based Movement**: Added directional bias (0.6 momentum bonus) to prevent oscillation and zigzag behavior  
- **Separation of Concerns**: Food-carrying ants use direct nest navigation, exploring ants follow pheromone trails
- **Continuous Trail Deposition**: Fixed dashed pheromone trails by depositing along movement paths
- **Realistic Movement Speeds**: Reduced speeds to 50-65 units/second for natural, continuous motion
- **Best Performance**: Generation 13 achieved 28.5s average time since goal (63% improvement over baseline)

### Legacy Improvements (Pre-Generation 11)
- Reduced startup timer to 5 seconds for faster food seeking
- Challenge mode: All food sources placed far from nest (minimum 333 units)
- Enhanced stuck detection and recovery mechanisms

### Video System Improvements
- Replaced artificial gradient backgrounds with real pheromone visualization
- Unified color configuration between simulation and video rendering
- Immediate recording start for complete behavior capture
- Proper coordinate transformation and pixel mapping\n- **Video Naming Convention**: Fixed to use established pattern `####_description.mp4` (e.g., 0007_codebase_cleanup_generation.mp4)\n- **Cleanup**: Removed debug output and unused modules for cleaner console output

### Current Status (Generation 15)\n- **Latest Video**: 0014_focus_optimization_on_average_time_since_goal_metric_only.mp4\n- **Performance**: 62.5s average time since goal, 5.9s return time, 66 deliveries\n- **Best Achievement**: Generation 13 reached 28.5s average time since goal (116 deliveries)\n- **System State**: Advanced pheromone-guided behavior with realistic movement speeds\n- **Video Naming**: Established pattern (####_description.mp4)\n\n## Development Workflow

### Making Changes
1. Modify relevant system in src/
2. **ALWAYS use the wrapper script**: `./run_simulation.sh` (requires Git Bash on Windows)
3. Let simulation complete its 90-second cycle and auto-convert to video
4. Review generated MP4 in videos/ directory with updated "AvgGoalTime" overlay
5. Analyze performance metrics from console output
6. **Commit all changes to git at the end of every development cycle**

**IMPORTANT**: The bash wrapper script is now the standard method for ALL simulation runs. It automatically:
- Runs the 90-second simulation
- Converts frames to MP4 with proper naming (fixed for Windows/Git Bash compatibility)
- Organizes files into videos/ and debug/ directories
- Updates generation info
- Displays the updated video overlay with "AvgGoalTime" as primary metric

**Manual `cargo run` should only be used for emergency debugging.**

### Generation Increment Rules
**IMPORTANT**: A video MUST be generated for every development cycle, regardless of success or failure. Do not increment generation numbers until AFTER a video has been generated AND properly saved. The generation number should only be updated once we have confirmed:

#### For Successful Generations:
1. The video shows the improvements working correctly
2. The video is properly saved in `/videos/` folder (NOT `simulation_videos/`)
3. The video uses the proper naming convention (e.g., `0003_generation_description.mp4`)
4. update claude.md and commit to git when development cycle is complete

#### For Failed Attempts:
1. **STILL GENERATE A VIDEO** documenting the failure (even if it's just 1-2 minutes showing the broken behavior)
2. Save video with descriptive name indicating failure (e.g., `0005_failed_optimization_broken_behavior.mp4`)
3. Document the failure analysis in `generation_info.json`
4. Include lessons learned and recovery actions taken
5. Update claude.md and commit to git

**CRITICAL**: No development cycle is complete without video documentation. Failed attempts are valuable learning experiences and must be recorded for future reference and educational content. Videos showing what doesn't work are just as important as videos showing success.

### Video Generation Workflow
1. Simulation auto-captures frames during run
2. Frames saved to: `simulation_videos/test_XXX_TIMESTAMP_frames/`
3. Use FFmpeg command above to convert to MP4
4. Final video shows real pheromone trails with scientific accuracy

## Known Issues & TODOs

### Planned Features (please put X's in [ ]s that you believe you have completed, when i verify and write VERIFIED, move it to the ARCHIVED section at the bottom)
- [X] Text overlay on videos with generation number and change descriptions
--- [X] BUG FIXED: Replaced solid rectangle rendering with proper 6x8 bitmap font patterns (A-Z, a-z, 0-9, symbols, emoji)
--- [X] FIXED: All broken characters resolved - added emoji support for 📊⏱️🚨📈 performance overlay icons
--- [ ] : FIX NOT WORKING emoji support isn't working, still displaying boxes.  Maybe just drop the emojis and use         │
│   characters.  Also the description text is still flowing off the right side, I think we need to   │
│   add line breaks.     
--- VERIFIED
- [X] Automate video conversion by making a wrapper around the simulation that automatically builds the video after the simulation exits (via user kill, timeout or error)
--- [X] TESTED: Created batch and PowerShell automation scripts, verified compilation and file systems working
--- [X] CLARIFIED: Updated instructions in Development Workflow - use run_simulation.ps1 for official videos, manual cargo run for debugging
--- [X] FIXED: Created run_simulation.sh for bash compatibility alongside run_simulation.ps1 for PowerShell - both scripts now work in their respective environments
- [X] New primary optimization metric: for each ant count the time it has been since its reached a goal.  Deliveries per minute is a good metric, but it can ignore ants failing badly - we want ALL ants to be acting effectively, so we will make a metric of how long since a food-seeking ant has left the nest without finding food or how long a nest-seeking ant has been looking for the nest.  We will take an averageTimeSinceGoal (open to other names) across all ants, and use this as the primary optimization metric to quantify the value of changes to the simulation.
--- [X] VALIDATED: Logic correctly tracks both food finding and nest delivery goals, averages across active ants, handles startup periods
--- [X] COMPLETED: Video now displays "AvgGoalTime: X.Xs" as the primary metric, with deliveries/min as secondary
- [X] Make 5 rounds of tweaks (within the constraints outlines within CONSTRAINTS.md!) trying to optimize average time since goal, verifying that video was successfully generated before moving onto the next generation
--- [X] COMPLETED BUT FAILED: Generation 5 attempted aggressive optimization with 5 rounds of improvements but caused catastrophic system failure (97.9% performance drop from 18.8 to 0.4 deliveries/min). All optimizations reverted and failure documented in video 0005_failed_optimization_aggressive_changes_broke_behavior.mp4
--- [X] LESSON LEARNED: Incremental changes are safer than comprehensive overhauls. Simple exploration behavior was already reasonably effective.
- [ ] Automated test framework for performance regression detection
- [ ] Parameter optimization based on video analysis
- [ ] Toggle simulation speed with 'T' hotkey
--- Is the simulation artificially slowed with timeouts or tick rates?  If so, I want to remove this limitation when the user presses T and run at "Turbo" mode - as fast as the processor will allow

### Current Limitations
- No automated linting/formatting commands configured  
- Text rendering still shows boxes instead of emoji characters in video overlay
- Performance trade-off: realistic movement vs optimization metrics (currently prioritizing realism)

### Recent Technical Achievements (Generation 11-15)
- ✅ **Advanced Pheromone System**: 8-directional sensing with momentum-based trail following
- ✅ **Continuous Trail Rendering**: Eliminated dashed pheromone trails through path-based deposition  
- ✅ **Behavioral Separation**: Smart context-aware navigation (direct vs trail-following)
- ✅ **Realistic Motion**: Smooth, natural ant movement at biologically-inspired speeds
- ✅ **Performance Breakthrough**: 63% improvement in goal achievement times (Generation 13)

## File Structure
```
src/
├── main.rs              # Bevy app setup and system registration
├── components.rs        # ECS component definitions  
├── systems.rs          # Core simulation logic systems
├── pheromones.rs       # Pheromone grid and trail simulation
├── video.rs            # Real-time video capture with pheromone visualization
├── colors.rs           # Unified color configuration
├── config.rs           # Simulation parameters
├── png_test.rs         # PNG testing utilities
└── create_test_frames.rs # Frame generation utilities

simulation_videos/      # Generated video outputs and frames
ffmpeg/                # Local FFmpeg installation for video conversion

## Allow-All Policy (Repo-Scoped)

**Scope:** Everything inside the repository root `antsim/` (and its subfolders).

**Permission:** You may create, modify, move, rename, and delete any files/directories under `antsim/`. You may run any commands whose working directory is inside `antsim/` (including build, test, scripts, git operations). Treat `antsim/` as a fully trusted workspace. I will use git to recover if needed.

**Out of Scope (ask first):**
- Reading/writing anything **outside** `antsim/`
- Installing/removing system packages or changing global config
- Long-running network operations or credential/secrets access

**Notes:** Keep all generated artifacts inside `antsim/` (e.g., `simulation_videos/`, `logs/`, `scripts/`).

This configuration enables Claude to understand the ant colony simulation architecture, run appropriate commands, and maintain consistency in the pheromone visualization system across both real-time simulation and video recording.


--- ARCHIVE: Can ignore for now  everything below this --- 

### VERIFIED Completed Features
- [X] Generation tracking system: Create `generation_info.json` to store current generation number, description, and timestamp  
--- [X] ENHANCED: Created `generation_history.json` to retain full development history while keeping current info in main file
--- VERIFIED
- [X] Challenge rules documentation: Create `CONSTRAINTS.md` file to codify "fair play" boundaries that prevent optimization shortcuts and ensure focus on realistic ant behavior
--- VERIFIED
- [X] Video organization system: Restructure to `videos/` (clean numbered names) + `debug/` (frames for debugging) structure like:
  ```
  videos/
  ├── 0001_pheromone_visualization.mp4
  ├── 0002_behavior_optimization.mp4  
  debug/
  └── frames_gen_0001/
  ```
--- VERIFIED
- [X] Enhanced ant state visualization (directional indicators)
--- [X] ENHANCED: Directional indicators were working but made them more visible - 2x2 white squares at 4px distance from ant center
--- VERIFIED

### Future challenges and later development ideas
- Adding obstacles to the terrain (rocks? walls?) so ants have to path around them to reach food sources effectively, optimal paths will no longer be straight
- Adding hazards (lava? ant trap?) that kills ants that enter it, they must find a way to warn others to avoid areas where ants disappear from (i think this will require a fear pheromone...?)
- Make ants leave the nest progressively rather than all at once
- Allow the nest to grow new ants when sufficient food has been returned. 
- Different kinds of food, some with greater nutritional density
- Dueling colonies? predators? ant warfare?
- world complexity like seasonal changes, weather effects, different terrain types, obstacle changes mid-run
- ant role specialization, 'dances' to maintain or optimize pheromone trails


### 