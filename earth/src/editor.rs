use bevy::{
    app::{
        AppExit,
        PluginGroupBuilder,
    },
    ui::FocusPolicy,
    prelude::*,
};

use earth::{
    ClearGrid,
    city::AddCity,
    nature::AddForest,
    ocean::AddOcean,
    rng::{SaveSeed, LoadSeed},
    generation::ScheduleGenerate
};

use bevytest::prelude::*;

use std::{
    fs,
    path::PathBuf,
};

const BUFFER_LENGTH: usize = 1024;

pub struct EditorPlugins;

impl PluginGroup for EditorPlugins {
    fn build(self) -> PluginGroupBuilder {
        let settings = SceneSettings {
            cameras: CameraSettings::AzimuthElevation(
                AzimuthElevationSettings::high_angle()
            ),
            lighting: LightSettings::DayNightCycle(
                DayNightCycleSettings::default()
            ),
        };

        PluginGroupBuilder::start::<Self>()
            .add(ConsolePlugin)
            .add(ScenePlugin::with_settings(settings))
    }
}

#[derive(Clone, Resource)]
struct ConsoleTextStyles {
    info: TextStyle,
    _warning: TextStyle,
    error: TextStyle,
    prompt: TextStyle,
    input: TextStyle,
    _argument_name: TextStyle,
    _argument_value: TextStyle,
}

#[derive(Resource, Clone, Debug)]
struct ConsoleCommandBuffer(String);
#[derive(Resource, Clone, Debug)]
struct ConsoleHistoryBuffer(Vec<String>);

#[derive(Clone, Debug)]
struct ExecutionRequested(String);

struct ConsolePlugin;

fn insert_text_styles(mut commands: Commands, assets: Res<AssetServer>) {
    let font = assets.load("fonts/source_code_pro/SourceCodePro-Regular.otf");
    let size = 16.0; // 16px font size

    let normal = TextStyle {
        font: font.clone(),
        font_size: size,
        color: Color::WHITE,
    };

    let prompt = TextStyle {
        font: font.clone(),
        font_size: size + 4.0, // slightly larger prompt
        color: Color::INDIGO,
    };

    let warning = TextStyle {
        font: font.clone(),
        font_size: size,
        color: Color::YELLOW,
    };

    let error = TextStyle {
        font: font.clone(),
        font_size: size,
        color: Color::RED,
    };

    let argument_name = TextStyle {
        font,
        font_size: size,
        color: Color::TURQUOISE,
    };

    commands.insert_resource(ConsoleTextStyles {
        info: normal.clone(),
        _warning: warning,
        error,
        prompt,
        input: normal.clone(),
        _argument_name: argument_name,
        _argument_value: normal,
    });
}

fn insert_buffers(mut commands: Commands) {
    commands.insert_resource(ConsoleCommandBuffer(String::new()));
    commands.insert_resource(ConsoleHistoryBuffer(Vec::new()));
}

#[derive(Component)]
struct Console;
#[derive(Component)]
struct ConsoleLog;
#[derive(Component)]
struct ConsolePrompt;

fn spawn_log_area(builder: &mut ChildBuilder<'_, '_, '_>) {
    builder
        .spawn(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::FlexEnd,
                flex_shrink: 1.0,
                flex_grow: 1.0,
                ..default()
            },
            ..default()
        })
        .insert(ConsoleLog)
        .insert(Name::new("Console Log"));
}

fn spawn_prompt_area(builder: &mut ChildBuilder<'_, '_, '_>, styles: &ConsoleTextStyles) {
    let line_height = styles.prompt.font_size.max(styles.input.font_size);

    builder
        .spawn(TextBundle {
            text: Text::from_sections([
                TextSection::new("→ ", styles.prompt.clone()),
                TextSection::from_style(styles.input.clone()),
            ]),
            style: Style {
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::FlexStart,
                flex_grow: 0.0,
                flex_shrink: 0.0,
                size: Size::height(Val::Px(line_height + 8.0)),
                ..default()
            },
            focus_policy: FocusPolicy::Block,
            ..default()
        })
        .insert(ConsolePrompt)
        .insert(Name::new("Console Input Line"));
}

fn spawn_console(mut commands: Commands, styles: Res<ConsoleTextStyles>) {
    commands
        .spawn(NodeBundle{
            background_color: BackgroundColor(Color::rgba(0.0, 0.0, 0.0, 0.8)),
            style: Style {
                display: Display::None,
                flex_direction: FlexDirection::ColumnReverse,
                justify_content: JustifyContent::FlexStart,
                size: Size::width(Val::Percent(100.0)),
                max_size: Size::height(Val::Percent(30.0)),
                ..default()
            },
            ..default()
        })
        .with_children(|builder| {
            spawn_prompt_area(builder, styles.as_ref());
            spawn_log_area(builder);
        })
        .insert(Console)
        .insert(Name::new("Console"));
}

fn toggle_console(mut input: ResMut<Input<KeyCode>>, mut query: Query<&mut Style, With<Console>>) {
    if input.just_released(KeyCode::Grave) {
        input.clear_just_released(KeyCode::Grave);
        let display = &mut query.single_mut().display;
        *display = match display {
            Display::Flex => Display::None,
            Display::None => Display::Flex,
        }
    }
}

fn console_open(query: Query<&Style, With<Console>>) -> bool {
    matches!(query.single().display, Display::Flex)
}

impl Plugin for ConsolePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ExecutionRequested>()
            .add_startup_systems((insert_text_styles, insert_buffers).in_base_set(StartupSet::PreStartup))
            .add_startup_system(spawn_console)
            .add_system(toggle_console)
            .add_system(edit_buffer.run_if(console_open))
            .add_system(update_console.run_if(console_open))
            .add_system(execute_commands);
    }
}

fn execute_commands(
    mut commands: Commands,
    log: Query<Entity, With<ConsoleLog>>,
    mut history: ResMut<ConsoleHistoryBuffer>,
    mut exit: EventWriter<AppExit>,
    mut execution_requests: EventReader<ExecutionRequested>,
    styles: Res<ConsoleTextStyles>,
) {
    for request in execution_requests.into_iter() {

        let input = &request.0;
        history.0.push(input.clone());

        let result = try_command(&mut commands, &mut exit, input);
        let (text, style) = match result {
            Ok(result) => (result, styles.info.clone()),
            Err(result) => (format!("{}", result), styles.error.clone())
        };

        let log_line = commands.spawn(TextBundle::from_section(text.clone(), style)).id();
        let mut log_commands = commands.entity(log.single());
        log_commands.add_child(log_line);
        info!("{}", text);
    }
}

struct CommandParseError(String);

impl std::fmt::Display for CommandParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "command parsing error: {}", self.0)
    }
}

type CommandParseResult = Result<String, CommandParseError>;

fn try_command(commands: &mut Commands, exit: &mut EventWriter<AppExit>, command: &str) -> CommandParseResult {
    let command = command.trim();
    if command.is_empty() {
        return Err(CommandParseError("no command given".into()));
    }

    let mut words = command.split_whitespace();
    let command_name = words.next().unwrap();

    match command_name {
        "add" => try_add_command(commands, words), // Pass the remaining args
        "clear" => {
            commands.add(ClearGrid);
            Ok("clearing grid".into())
        },
        "quit"|"q"|"exit" => {
            exit.send(AppExit);
            Ok("exiting…".into())
        },
        "save" =>
            schedule_seed_save(commands, words.next().unwrap_or("./seed")),
        "load" =>
            schedule_seed_load(commands, words.next().unwrap_or("./seed")),
        "generate" => {
            commands.add(ScheduleGenerate);
            Ok("generating map…".into())
        },
        _ => Err(CommandParseError(format!("unknown command: \"{command_name}\""))),
    }
}

fn schedule_seed_save(commands: &mut Commands, path_str: &str) -> CommandParseResult {
    let file_path = PathBuf::from(path_str);
    let file = fs::File::options()
        .write(true)
        .create_new(true)
        .open(file_path.clone())
        .map_err(|e| CommandParseError(format!("cannot create seed file \"{}\": {}", file_path.display(), e)))?;

    let absolute_path = file_path
        .canonicalize()
        .map_err(|e| CommandParseError(format!("could not find absolute path: {e}")))?;

    commands.add(SaveSeed { file });
    Ok(format!("saving seed to {}", absolute_path.display()))
}

fn schedule_seed_load(commands: &mut Commands, path_str: &str) -> CommandParseResult {
    let file_path = PathBuf::from(path_str);
    let file = fs::File::options()
        .read(true)
        .open(file_path.clone())
        .map_err(|e| CommandParseError(format!("cannot open seed file \"{}\": {}", file_path.display(), e)))?;



    let absolute_path = file_path
        .canonicalize()
        .map_err(|e| CommandParseError(format!("could not determine absolute seed path: {e}")))?;

    commands.add(LoadSeed { file });
    Ok(format!("loading seed from {}", absolute_path.display()))
}

fn try_add_command<'a, I>(
    commands: &mut Commands,
    arguments: I,
) -> CommandParseResult where
    I: IntoIterator<Item = &'a str>
{
    let mut arguments = arguments.into_iter();
    let biome_name = arguments.next();
    if biome_name.is_none() {
        return Err(CommandParseError(
            "no biome given, options are: city|forest|ocean".into(),
        ));
    }

    let biome_name = biome_name.unwrap();

    let arguments = arguments.collect::<Vec<&str>>();
    
    match biome_name {
        "city" => {
            let command = AddCity::try_from(arguments)
                .map_err(|e| CommandParseError(format!("{}", e)))?;
            commands.add(command);
            Ok("city added".to_string())
        },
        "forest" => {
            let command = AddForest::try_from(arguments)
                .map_err(|e| CommandParseError(format!("{}", e)))?;
            commands.add(command);
            Ok("forest added".to_string())
        },
        "ocean" => {
            let command = AddOcean::try_from(arguments)
                .map_err(|e| CommandParseError(format!("{}", e)))?;
            commands.add(command);
            Ok("ocean added".to_string())
        },
        _ => Err(CommandParseError(format!("biome not supported: {}", biome_name)))
    }                
}

fn edit_buffer(
    mut input_characters: EventReader<ReceivedCharacter>,
    mut buffer: ResMut<ConsoleCommandBuffer>,
    mut execution_request: EventWriter<ExecutionRequested>,
) {
    for c in input_characters.iter().map(|c| c.char) {
        if c == '\x08' { // Backspace
            buffer.0.pop();
            return;
        } else if c == '\x0D' { // Enter
            let command: String = buffer.0.drain(..).collect();
            execution_request.send(ExecutionRequested(command));
            return;
        }

        // Ensure input does not overflow buffer length
        if buffer.0.len() + c.len_utf8() > BUFFER_LENGTH { return; }

        if c != '`' && c.is_ascii_graphic() {
            buffer.0.push(c);
        } else if c.is_whitespace() {
            // Push all whitespace as "standard" ASCII #32,
            // a.k.a. the space character.
            buffer.0.push(' ');
        }
    }

    input_characters.clear()
}

fn update_console(buffer: Res<ConsoleCommandBuffer>, mut input_display: Query<&mut Text, With<ConsolePrompt>>) {
    if !buffer.is_changed() { return }
    let command_text = &mut input_display.single_mut().sections[1];
    command_text.value = buffer.0.clone();
}
