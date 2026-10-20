use crate::mapping::block_state::block_static;
use crate::mapping::client::camera::Camera;
use crate::mapping::client::world::World;
use crate::mapping::entity::mob::Mob;
use crate::mapping::entity::player::Player;
use crate::mapping::entity::{Entity, EntityRef, LivingEntityRef};
use crate::mapping::math::{BlockPos, Vec3};
use crate::mapping::MappedObject;
use crate::module::{ModuleId, ModuleSetting};
use crate::state::{client, mapping, minecraft};
use egui::{
    pos2, vec2, Align2, Color32, Context, FontId, Id, LayerId, Order, Painter, Pos2, Rect,
    Rounding, Stroke,
};
use jni::objects::GlobalRef;
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

const GATHER_INTERVAL: Duration = Duration::from_millis(66);

const CHEST_SCAN_INTERVAL: Duration = Duration::from_secs(2);

const CHEST_CHUNK_RADIUS: i32 = 8;

const BLOCK_SCAN_INTERVAL: Duration = Duration::from_secs(2);
const BLOCK_SCAN_RADIUS: i32 = 6;
const XRAY_BLOCKS_PER_GATHER: usize = 512;
const XRAY_SCAN_INTERVAL: Duration = Duration::from_secs(2);

const LINE_WIDTH: f32 = 1.6;

const ORE_BLOCKS: &[(&str, &str)] = &[
    ("COAL_ORE", "Coal Ore"),
    ("DEEPSLATE_COAL_ORE", "Deepslate Coal Ore"),
    ("IRON_ORE", "Iron Ore"),
    ("DEEPSLATE_IRON_ORE", "Deepslate Iron Ore"),
    ("COPPER_ORE", "Copper Ore"),
    ("DEEPSLATE_COPPER_ORE", "Deepslate Copper Ore"),
    ("GOLD_ORE", "Gold Ore"),
    ("DEEPSLATE_GOLD_ORE", "Deepslate Gold Ore"),
    ("NETHER_GOLD_ORE", "Nether Gold Ore"),
    ("REDSTONE_ORE", "Redstone Ore"),
    ("DEEPSLATE_REDSTONE_ORE", "Deepslate Redstone Ore"),
    ("LAPIS_ORE", "Lapis Lazuli Ore"),
    ("DEEPSLATE_LAPIS_ORE", "Deepslate Lapis Lazuli Ore"),
    ("DIAMOND_ORE", "Diamond Ore"),
    ("DEEPSLATE_DIAMOND_ORE", "Deepslate Diamond Ore"),
    ("EMERALD_ORE", "Emerald Ore"),
    ("DEEPSLATE_EMERALD_ORE", "Deepslate Emerald Ore"),
    ("NETHER_QUARTZ_ORE", "Nether Quartz Ore"),
    ("ANCIENT_DEBRIS", "Ancient Debris"),
];

#[derive(Debug, Clone, Copy)]
struct V3 {
    x: f64,
    y: f64,
    z: f64,
}

impl V3 {
    fn sub(self, o: V3) -> V3 {
        V3 {
            x: self.x - o.x,
            y: self.y - o.y,
            z: self.z - o.z,
        }
    }

    fn dot(self, o: V3) -> f64 {
        self.x * o.x + self.y * o.y + self.z * o.z
    }

    fn cross(self, o: V3) -> V3 {
        V3 {
            x: self.y * o.z - self.z * o.y,
            y: self.z * o.x - self.x * o.z,
            z: self.x * o.y - self.y * o.x,
        }
    }

    fn lerp(self, o: V3, t: f64) -> V3 {
        V3 {
            x: self.x + (o.x - self.x) * t,
            y: self.y + (o.y - self.y) * t,
            z: self.z + (o.z - self.z) * t,
        }
    }

    fn length(self) -> f64 {
        self.dot(self).sqrt()
    }
}

impl From<Vec3> for V3 {
    fn from(v: Vec3) -> V3 {
        V3 {
            x: v.x(),
            y: v.y(),
            z: v.z(),
        }
    }
}

struct View {
    cam: V3,
    fwd: V3,
    right: V3,
    up: V3,
    tan_x: f64,
    tan_y: f64,
    w: f32,
    h: f32,
}

impl View {
    fn project(&self, p: V3) -> Option<Pos2> {
        let r = p.sub(self.cam);
        let depth = r.dot(self.fwd);
        if depth < 0.05 {
            return None;
        }
        let ndc_x = (r.dot(self.right) / depth) / self.tan_x;
        let ndc_y = (r.dot(self.up) / depth) / self.tan_y;
        let sx = (ndc_x * 0.5 + 0.5) * self.w as f64;
        let sy = (0.5 - ndc_y * 0.5) * self.h as f64;
        Some(pos2(sx as f32, sy as f32))
    }
}

fn build_view(cam: V3, yaw_deg: f32, pitch_deg: f32, fov_deg: f64, w: f32, h: f32) -> View {
    let yaw = (yaw_deg as f64).to_radians();
    let pitch = (pitch_deg as f64).to_radians();
    let (sy, cy) = yaw.sin_cos();
    let (sp, cp) = pitch.sin_cos();

    let fwd = V3 {
        x: -sy * cp,
        y: -sp,
        z: cy * cp,
    };
    let right = V3 {
        x: -cy,
        y: 0.0,
        z: -sy,
    };
    let up = right.cross(fwd);

    let tan_y = (fov_deg.to_radians() * 0.5).tan();
    let tan_x = tan_y * (w / h) as f64;

    View {
        cam,
        fwd,
        right,
        up,
        tan_x,
        tan_y,
        w,
        h,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TargetKind {
    Player,
    Mob,
}

#[derive(Debug, Clone)]
struct EntityTarget {
    id: i32,
    kind: TargetKind,
    uuid: Option<String>,
    identity_name: String,

    prev: V3,

    pos: V3,
    width: f64,
    height: f64,
    name: String,
    health: f32,
    max_health: f32,
}

#[derive(Debug, Clone, Copy)]
struct ChestTarget {
    pos: V3,
}

#[derive(Debug, Clone)]
struct MineralTarget {
    pos: V3,
    name: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ScannedBlock {
    Air,
    SafeMaterial,
    Other,
}

#[derive(Debug, Clone, Copy)]
struct HoleTarget {
    pos: V3,
}

struct EspState {
    camera: Option<Camera>,
    entities: Vec<EntityTarget>,
    chests: Vec<ChestTarget>,
    minerals: Vec<MineralTarget>,
    holes: Vec<HoleTarget>,
    prev_gather: Option<Instant>,
    last_gather: Option<Instant>,
    last_chest_scan: Option<Instant>,
    last_block_scan: Option<Instant>,
    xray_scan: Option<XrayScan>,
    last_xray_scan: Option<Instant>,

    fov: f64,

    target_fov: f64,

    camera_logged: bool,
}

impl EspState {
    fn new() -> Self {
        EspState {
            camera: None,
            entities: Vec::new(),
            chests: Vec::new(),
            minerals: Vec::new(),
            holes: Vec::new(),
            prev_gather: None,
            last_gather: None,
            last_chest_scan: None,
            last_block_scan: None,
            xray_scan: None,
            last_xray_scan: None,
            fov: 70.0,
            target_fov: 70.0,
            camera_logged: false,
        }
    }
}

fn state() -> &'static Mutex<EspState> {
    static STATE: OnceLock<Mutex<EspState>> = OnceLock::new();
    STATE.get_or_init(|| Mutex::new(EspState::new()))
}

// --- module configuration --------------------------------------------------

/// Per-category drawing options, snapshotted from the modules once per frame.
struct EntityCfg {
    enabled: bool,
    color: Color32,
    show_name: bool,
    show_distance: bool,
    show_health: bool,
    ignore_friends: bool,
    /// Maximum distance, in blocks, an entity is drawn (and processed) at.
    range: f32,
}

struct ChestCfg {
    enabled: bool,
    color: Color32,
    show_distance: bool,
    hole_enabled: bool,
    hole_color: Color32,
    hole_range: f32,
    mineral_enabled: bool,
    mineral_color: Color32,
    mineral_range: f32,
    mineral_vertical_range: f32,
    block_range: f32,
}

struct XrayScan {
    world: GlobalRef,
    center: (i32, i32, i32),
    radius: i32,
    vertical_range: i32,
    cursor: usize,
    total: usize,
    ore_blocks: Vec<(GlobalRef, &'static str)>,
    found: Vec<MineralTarget>,
}

/// Tracer-line options, snapshotted from the Tracers module once per frame.
struct TracerCfg {
    enabled: bool,
    color: Color32,
    players: bool,
    mobs: bool,
    /// Maximum distance, in blocks, a tracer is drawn at.
    range: f32,
}

/// Name-tag options, snapshotted from the Nametags module once per frame.
struct NametagCfg {
    enabled: bool,
    color: Color32,
    show_health: bool,
    show_distance: bool,
    ignore_friends: bool,
    /// Maximum distance, in blocks, a name tag is drawn at.
    range: f32,
}

struct EspConfig {
    player: EntityCfg,
    mob: EntityCfg,
    chest: ChestCfg,
    tracer: TracerCfg,
    nametag: NametagCfg,
}

impl EspConfig {
    fn any_enabled(&self) -> bool {
        self.player.enabled
            || self.mob.enabled
            || self.chest.enabled
            || self.chest.hole_enabled
            || self.chest.mineral_enabled
            || self.tracer.enabled
            || self.nametag.enabled
    }

    /// Whether players must be gathered — wanted by Player ESP, by tracers, or
    /// by name tags.
    fn want_players(&self) -> bool {
        self.player.enabled || (self.tracer.enabled && self.tracer.players) || self.nametag.enabled
    }

    /// Whether mobs must be gathered — wanted by Mob ESP or by tracers.
    fn want_mobs(&self) -> bool {
        self.mob.enabled || (self.tracer.enabled && self.tracer.mobs)
    }
}

fn color_setting(data: &crate::module::ModuleData, name: &str, fallback: Color32) -> Color32 {
    match data.get_setting(name) {
        Some(ModuleSetting::Color { value, .. }) => Color32::from_rgba_unmultiplied(
            (value[0] * 255.0) as u8,
            (value[1] * 255.0) as u8,
            (value[2] * 255.0) as u8,
            (value[3] * 255.0).max(40.0) as u8,
        ),
        _ => fallback,
    }
}

fn toggle_setting(data: &crate::module::ModuleData, name: &str, fallback: bool) -> bool {
    data.get_setting(name)
        .and_then(ModuleSetting::get_toggle_value)
        .unwrap_or(fallback)
}

fn slider_setting(data: &crate::module::ModuleData, name: &str, fallback: f32) -> f32 {
    data.get_setting(name)
        .and_then(ModuleSetting::get_slider_value)
        .unwrap_or(fallback)
}

/// Reads the three ESP modules' state with a single lock pass.
fn read_config() -> EspConfig {
    let disabled_entity = || EntityCfg {
        enabled: false,
        color: Color32::WHITE,
        show_name: false,
        show_distance: false,
        show_health: false,
        ignore_friends: false,
        range: 0.0,
    };
    let mut cfg = EspConfig {
        player: disabled_entity(),
        mob: disabled_entity(),
        chest: ChestCfg {
            enabled: false,
            color: Color32::WHITE,
            show_distance: false,
            hole_enabled: false,
            hole_color: Color32::WHITE,
            hole_range: 0.0,
            mineral_enabled: false,
            mineral_color: Color32::WHITE,
            mineral_range: 0.0,
            mineral_vertical_range: 0.0,
            block_range: 0.0,
        },
        tracer: TracerCfg {
            enabled: false,
            color: Color32::WHITE,
            players: false,
            mobs: false,
            range: 0.0,
        },
        nametag: NametagCfg {
            enabled: false,
            color: Color32::WHITE,
            show_health: false,
            show_distance: false,
            ignore_friends: false,
            range: 0.0,
        },
    };

    let modules = &client().modules;

    if let Some(arc) = modules.get(ModuleId::PlayerEsp) {
        if let Ok(module) = arc.lock() {
            let data = module.get_module_data();
            cfg.player = EntityCfg {
                enabled: data.enabled,
                color: color_setting(data, "Color", Color32::from_rgb(255, 70, 70)),
                show_name: toggle_setting(data, "Name", true),
                show_distance: toggle_setting(data, "Distance", true),
                show_health: toggle_setting(data, "Health", true),
                ignore_friends: toggle_setting(data, "Ignore Friends", false),
                range: slider_setting(data, "Range", 64.0),
            };
        }
    }
    if let Some(arc) = modules.get(ModuleId::MobEsp) {
        if let Ok(module) = arc.lock() {
            let data = module.get_module_data();
            cfg.mob = EntityCfg {
                enabled: data.enabled,
                color: color_setting(data, "Color", Color32::from_rgb(255, 215, 50)),
                show_name: toggle_setting(data, "Name", true),
                show_distance: toggle_setting(data, "Distance", true),
                show_health: toggle_setting(data, "Health", true),
                ignore_friends: false,
                range: slider_setting(data, "Range", 64.0),
            };
        }
    }
    if let Some(arc) = modules.get(ModuleId::ChestEsp) {
        if let Ok(module) = arc.lock() {
            let data = module.get_module_data();
            cfg.chest = ChestCfg {
                enabled: data.enabled && toggle_setting(data, "Containers", true),
                color: color_setting(data, "Color", Color32::from_rgb(255, 140, 30)),
                show_distance: toggle_setting(data, "Distance", true),
                hole_enabled: false,
                hole_color: Color32::from_rgb(50, 230, 115),
                hole_range: 0.0,
                mineral_enabled: false,
                mineral_color: Color32::from_rgb(50, 200, 255),
                mineral_range: 0.0,
                mineral_vertical_range: 0.0,
                block_range: 0.0,
            };
        }
    }
    if let Some(arc) = modules.get(ModuleId::HoleEsp) {
        if let Ok(module) = arc.lock() {
            let data = module.get_module_data();
            cfg.chest.hole_enabled = data.enabled;
            cfg.chest.hole_color = color_setting(data, "Color", Color32::from_rgb(50, 230, 115));
            cfg.chest.hole_range = slider_setting(data, "Range", 6.0).clamp(3.0, 6.0);
            cfg.chest.block_range = cfg.chest.block_range.max(cfg.chest.hole_range);
        }
    }
    if let Some(arc) = modules.get(ModuleId::MineralEsp) {
        if let Ok(module) = arc.lock() {
            let data = module.get_module_data();
            cfg.chest.mineral_enabled = data.enabled;
            cfg.chest.mineral_color = color_setting(data, "Color", Color32::from_rgb(50, 200, 255));
            cfg.chest.mineral_range = slider_setting(data, "Range", 12.0).clamp(3.0, 24.0);
            cfg.chest.mineral_vertical_range =
                slider_setting(data, "Vertical Range", 12.0).clamp(4.0, 24.0);
        }
    }
    if let Some(arc) = modules.get(ModuleId::Tracers) {
        if let Ok(module) = arc.lock() {
            let data = module.get_module_data();
            cfg.tracer = TracerCfg {
                enabled: data.enabled,
                color: color_setting(data, "Color", Color32::from_rgb(100, 200, 255)),
                players: toggle_setting(data, "Players", true),
                mobs: toggle_setting(data, "Mobs", false),
                range: slider_setting(data, "Range", 96.0),
            };
        }
    }
    if let Some(arc) = modules.get(ModuleId::Nametags) {
        if let Ok(module) = arc.lock() {
            let data = module.get_module_data();
            cfg.nametag = NametagCfg {
                enabled: data.enabled,
                color: color_setting(data, "Color", Color32::WHITE),
                show_health: toggle_setting(data, "Health", true),
                show_distance: toggle_setting(data, "Distance", false),
                ignore_friends: toggle_setting(data, "Ignore Friends", false),
                range: slider_setting(data, "Range", 96.0),
            };
        }
    }

    cfg
}

pub fn draw(ctx: &Context) {
    let middle_click = crate::graphic::input::take_middle_click();
    let cfg = read_config();
    let mut state = state().lock().unwrap();

    if middle_click {
        toggle_crosshair_friend(&mut state, ctx);
    }

    if !cfg.any_enabled() {
        state.entities.clear();
        state.chests.clear();
        state.minerals.clear();
        state.holes.clear();
        state.xray_scan = None;
        state.last_xray_scan = None;
        return;
    }

    let now = Instant::now();
    if state.last_gather.is_none_or(|t| now - t >= GATHER_INTERVAL) {
        gather(&mut state, &cfg, now);
    }

    let view = match read_view(&mut state, ctx) {
        Some(view) => view,
        None => return,
    };
    let t = interp_factor(&state, now);

    let painter = ctx.layer_painter(LayerId::new(Order::Background, Id::new("esp_overlay")));

    for entity in &state.entities {
        draw_entity(&painter, &view, entity, t, &cfg);
        if cfg.tracer.enabled {
            draw_tracer(&painter, &view, entity, t, &cfg.tracer);
        }
        if cfg.nametag.enabled {
            draw_nametag(&painter, &view, entity, t, &cfg.nametag);
        }
    }
    for chest in &state.chests {
        draw_chest(&painter, &view, chest, &cfg);
    }
    for hole in &state.holes {
        draw_hole(&painter, &view, hole, cfg.chest.hole_color);
    }
    for mineral in &state.minerals {
        if mineral.pos.sub(view.cam).length() > cfg.chest.mineral_range as f64 + 1.0 {
            continue;
        }
        draw_mineral(&painter, &view, mineral, cfg.chest.mineral_color);
    }
}

fn interp_factor(state: &EspState, now: Instant) -> f64 {
    match (state.prev_gather, state.last_gather) {
        (Some(prev), Some(last)) => {
            let span = (last - prev).as_secs_f64();
            if span <= 1e-4 {
                1.0
            } else {
                ((now - last).as_secs_f64() / span).clamp(0.0, 1.0)
            }
        }
        _ => 1.0,
    }
}

fn read_view(state: &mut EspState, ctx: &Context) -> Option<View> {
    if state.camera.is_none() {
        match minecraft()
            .game_renderer()
            .and_then(|gr| gr.get_main_camera())
        {
            Ok(camera) => state.camera = Some(camera),
            Err(e) => {
                log::debug!("ESP: camera unavailable: {e}");
                return None;
            }
        }
    }

    let camera = state.camera.clone()?;
    let rect = ctx.screen_rect();
    if rect.width() < 1.0 || rect.height() < 1.0 {
        return None;
    }

    let read = (|| -> anyhow::Result<(V3, f32, f32)> {
        Ok((camera.position()?.into(), camera.yaw()?, camera.pitch()?))
    })();

    let (cam_pos, yaw, pitch) = match read {
        Ok(values) => values,
        Err(error) => {
            if !state.camera_logged {
                state.camera_logged = true;
                log::warn!("ESP: camera read failed: {error}");
            }
            return None;
        }
    };

    let fov_deg = resolve_fov(&camera, state, ctx);

    Some(build_view(
        cam_pos,
        yaw,
        pitch,
        fov_deg,
        rect.width(),
        rect.height(),
    ))
}

fn resolve_fov(camera: &Camera, state: &mut EspState, ctx: &Context) -> f64 {
    if let Ok(fov) = camera.fov() {
        if fov.is_finite() && fov > 0.0 {
            let snapped = (fov as f64).clamp(1.0, 179.0);
            state.fov = snapped;
            state.target_fov = snapped;
            return snapped;
        }
    }

    state.target_fov = formula_fov();
    let dt = ctx.input(|input| input.stable_dt).clamp(0.0, 0.1) as f64;
    let blend = 1.0 - 0.5_f64.powf(dt / 0.05);
    state.fov += (state.target_fov - state.fov) * blend;
    state.fov
}

fn formula_fov() -> f64 {
    let base = match read_option_fov() {
        Ok(fov) if fov.is_finite() && (1.0..=179.0).contains(&fov) => fov,
        _ => 70.0,
    };
    let raw = fov_modifier_raw();
    let scale = read_fov_effect_scale().unwrap_or(1.0).clamp(0.0, 1.0);
    let effective = 1.0 + (raw - 1.0) * scale;
    (base * effective).clamp(1.0, 179.0)
}

fn fov_modifier_raw() -> f64 {
    let player = match minecraft().player() {
        Ok(Some(player)) => player,
        _ => return 1.0,
    };

    let mut modifier = 1.0;
    if player.abilities.is_flying().unwrap_or(false) {
        modifier *= 1.1;
    }
    if player.is_sprinting().unwrap_or(false) {
        modifier *= 1.15;
    }
    modifier
}

fn read_option_fov() -> anyhow::Result<f64> {
    Ok(minecraft().options()?.fov()?.get_int()? as f64)
}

fn read_fov_effect_scale() -> anyhow::Result<f64> {
    minecraft().options()?.fov_effect_scale()?.get_double()
}

fn gather(state: &mut EspState, cfg: &EspConfig, now: Instant) {
    state.prev_gather = state.last_gather;
    state.last_gather = Some(now);

    let want_players = cfg.want_players();
    let want_mobs = cfg.want_mobs();
    if want_players || want_mobs {
        let mut range = 0.0_f32;
        if cfg.player.enabled {
            range = range.max(cfg.player.range);
        }
        if cfg.mob.enabled {
            range = range.max(cfg.mob.range);
        }
        if cfg.tracer.enabled {
            range = range.max(cfg.tracer.range);
        }
        if cfg.nametag.enabled {
            range = range.max(cfg.nametag.range);
        }
        let range_sq = (range as f64) * (range as f64);

        let previous = std::mem::take(&mut state.entities);
        match gather_entities(&previous, range_sq, want_players, want_mobs) {
            Ok(list) => state.entities = list,
            Err(e) => log::debug!("ESP: entity gather failed: {e}"),
        }
    } else {
        state.entities.clear();
    }

    if cfg.chest.enabled {
        let due = state
            .last_chest_scan
            .is_none_or(|t| now - t >= CHEST_SCAN_INTERVAL);
        if due {
            state.last_chest_scan = Some(now);
            match gather_chests() {
                Ok(list) => state.chests = list,
                Err(e) => log::debug!("ESP: chest scan failed: {e}"),
            }
        }
    } else {
        state.chests.clear();
        state.last_chest_scan = None;
    }

    if cfg.chest.hole_enabled {
        let due = state
            .last_block_scan
            .is_none_or(|t| now - t >= BLOCK_SCAN_INTERVAL);
        if due {
            state.last_block_scan = Some(now);
            match gather_block_targets(cfg) {
                Ok((holes, minerals)) => {
                    state.holes = holes;
                    state.minerals = minerals;
                }
                Err(e) => log::debug!("ESP: block scan failed: {e}"),
            }
        }
    } else {
        state.holes.clear();
        state.last_block_scan = None;
    }

    update_xray(state, cfg, now);
}

fn load_block_ref(name: &'static str) -> anyhow::Result<GlobalRef> {
    mapping().in_frame(|| {
        let block = block_static(name)?;
        mapping().new_global_ref(block)
    })
}

fn gather_block_targets(cfg: &EspConfig) -> anyhow::Result<(Vec<HoleTarget>, Vec<MineralTarget>)> {
    let (Some(world), Some(player)) = (minecraft().world()?, minecraft().player()?) else {
        return Ok((Vec::new(), Vec::new()));
    };
    let player_pos: V3 = player.get_position()?.into();
    let center = BlockPos::new(
        player_pos.x.floor() as i32,
        player_pos.y.floor() as i32,
        player_pos.z.floor() as i32,
    );
    let range = cfg.chest.block_range.min(BLOCK_SCAN_RADIUS as f32);
    let radius = range.ceil() as i32;
    let hole_range_sq = f64::from(cfg.chest.hole_range * cfg.chest.hole_range);
    let bedrock = block_static("BEDROCK")?;
    let obsidian = block_static("OBSIDIAN")?;

    let mut blocks = HashMap::new();
    for x in (center.x() - radius)..=(center.x() + radius) {
        for z in (center.z() - radius)..=(center.z() + radius) {
            let dx = x - center.x();
            let dz = z - center.z();
            if dx * dx + dz * dz > radius * radius {
                continue;
            }
            for y in (center.y() - 2)..=(center.y() + 2) {
                let pos = BlockPos::new(x, y, z);
                let state = world.get_block_state(&pos)?;
                let kind = if state.is_air()? {
                    ScannedBlock::Air
                } else {
                    let actual = state.get_block()?;
                    let env = mapping().get_env()?;
                    if env.is_same_object(actual.as_obj(), &bedrock)?
                        || env.is_same_object(actual.as_obj(), &obsidian)?
                    {
                        ScannedBlock::SafeMaterial
                    } else {
                        ScannedBlock::Other
                    }
                };
                blocks.insert((x, y, z), kind);
            }
        }
    }

    let mut holes = Vec::new();
    if cfg.chest.hole_enabled {
        for x in (center.x() - radius + 1)..=(center.x() + radius - 1) {
            for z in (center.z() - radius + 1)..=(center.z() + radius - 1) {
                for y in (center.y() - 1)..=center.y() {
                    let dx = x - center.x();
                    let dz = z - center.z();
                    if f64::from(dx * dx + dz * dz) > hole_range_sq {
                        continue;
                    }
                    if is_safe_hole(&blocks, x, y, z) {
                        holes.push(HoleTarget {
                            pos: V3 {
                                x: f64::from(x),
                                y: f64::from(y),
                                z: f64::from(z),
                            },
                        });
                    }
                }
            }
        }
    }

    Ok((holes, Vec::new()))
}

fn begin_xray_scan(
    world: &World,
    center: (i32, i32, i32),
    radius: i32,
    vertical_range: i32,
) -> anyhow::Result<XrayScan> {
    let mut ore_blocks = Vec::with_capacity(ORE_BLOCKS.len());
    for (field, label) in ORE_BLOCKS {
        match load_block_ref(field) {
            Ok(block) => ore_blocks.push((block, *label)),
            Err(error) => log::debug!("XRay: skipping unavailable block {field}: {error}"),
        }
    }
    if ore_blocks.is_empty() {
        anyhow::bail!("no supported ore blocks are available in this Minecraft version");
    }
    let width = (radius * 2 + 1) as usize;
    let height = (vertical_range * 2 + 1) as usize;
    Ok(XrayScan {
        world: world.jni_ref().clone(),
        center,
        radius,
        vertical_range,
        cursor: 0,
        total: width * width * height,
        ore_blocks,
        found: Vec::new(),
    })
}

fn update_xray(state: &mut EspState, cfg: &EspConfig, now: Instant) {
    if !cfg.chest.mineral_enabled {
        state.xray_scan = None;
        state.minerals.clear();
        state.last_xray_scan = None;
        return;
    }

    let (world, player) = match (minecraft().world(), minecraft().player()) {
        (Ok(Some(world)), Ok(Some(player))) => (world, player),
        (Ok(None), _) | (_, Ok(None)) => {
            state.xray_scan = None;
            state.minerals.clear();
            state.last_xray_scan = None;
            return;
        }
        (Err(error), _) | (_, Err(error)) => {
            log::debug!("XRay: could not access the current world: {error}");
            return;
        }
    };
    let player_pos = match player.get_position() {
        Ok(pos) => V3::from(pos),
        Err(error) => {
            log::debug!("XRay: could not read player position: {error}");
            return;
        }
    };
    let center = (
        player_pos.x.floor() as i32,
        player_pos.y.floor() as i32,
        player_pos.z.floor() as i32,
    );
    let radius = cfg.chest.mineral_range.ceil() as i32;
    let vertical_range = cfg.chest.mineral_vertical_range.ceil() as i32;
    let world_changed = if let Some(scan) = state.xray_scan.as_ref() {
        match mapping().get_env() {
            Ok(env) => match env.is_same_object(world.jni_ref().as_obj(), scan.world.as_obj()) {
                Ok(same_world) => !same_world,
                Err(error) => {
                    log::debug!("XRay: could not compare world identity: {error}");
                    return;
                }
            },
            Err(error) => {
                log::debug!("XRay: JNI environment unavailable: {error}");
                return;
            }
        }
    } else {
        false
    };
    let restart = world_changed
        || state.xray_scan.as_ref().is_some_and(|scan| {
            scan.radius != radius
                || scan.vertical_range != vertical_range
                || (scan.center.0 - center.0).abs() > (radius / 2).max(2)
                || (scan.center.1 - center.1).abs() > (vertical_range / 2).max(2)
                || (scan.center.2 - center.2).abs() > (radius / 2).max(2)
        });
    if restart {
        state.xray_scan = None;
        if world_changed {
            state.minerals.clear();
            state.last_xray_scan = None;
        }
    }

    if state.xray_scan.is_none()
        && state
            .last_xray_scan
            .is_none_or(|last| now - last >= XRAY_SCAN_INTERVAL)
    {
        match begin_xray_scan(&world, center, radius, vertical_range) {
            Ok(scan) => state.xray_scan = Some(scan),
            Err(error) => {
                state.last_xray_scan = Some(now);
                log::debug!("XRay: could not initialize ore scan: {error}");
                return;
            }
        }
    }

    let Some(mut scan) = state.xray_scan.take() else {
        return;
    };
    let mut checked = 0;

    while scan.cursor < scan.total && checked < XRAY_BLOCKS_PER_GATHER {
        let index = scan.cursor;
        scan.cursor += 1;

        let Some((x, y, z)) =
            xray_grid_position(scan.center, scan.radius, scan.vertical_range, index)
        else {
            continue;
        };
        checked += 1;

        let pos = BlockPos::new(x, y, z);
        let block_state = match world.get_block_state(&pos) {
            Ok(block_state) => block_state,
            Err(error) => {
                state.last_xray_scan = Some(now);
                log::debug!("XRay: block scan failed: {error}");
                return;
            }
        };
        match block_state.is_air() {
            Ok(true) => continue,
            Ok(false) => {}
            Err(error) => {
                state.last_xray_scan = Some(now);
                log::debug!("XRay: block state query failed: {error}");
                return;
            }
        }

        let actual = match block_state.get_block() {
            Ok(actual) => actual,
            Err(error) => {
                state.last_xray_scan = Some(now);
                log::debug!("XRay: could not resolve scanned block: {error}");
                return;
            }
        };
        let env = match mapping().get_env() {
            Ok(env) => env,
            Err(error) => {
                state.last_xray_scan = Some(now);
                log::debug!("XRay: JNI environment unavailable: {error}");
                return;
            }
        };
        for (ore, label) in &scan.ore_blocks {
            match env.is_same_object(actual.as_obj(), ore.as_obj()) {
                Ok(true) => {
                    scan.found.push(MineralTarget {
                        pos: V3 {
                            x: f64::from(pos.x()),
                            y: f64::from(pos.y()),
                            z: f64::from(pos.z()),
                        },
                        name: label,
                    });
                    break;
                }
                Ok(false) => {}
                Err(error) => {
                    state.last_xray_scan = Some(now);
                    log::debug!("XRay: ore comparison failed: {error}");
                    return;
                }
            }
        }
    }

    if scan.cursor >= scan.total {
        scan.found.sort_by(|a, b| {
            a.pos
                .sub(player_pos)
                .length()
                .total_cmp(&b.pos.sub(player_pos).length())
        });
        state.minerals = std::mem::take(&mut scan.found);
        state.last_xray_scan = Some(now);
    } else {
        state.xray_scan = Some(scan);
    }
}

fn xray_grid_position(
    center: (i32, i32, i32),
    radius: i32,
    vertical_range: i32,
    index: usize,
) -> Option<(i32, i32, i32)> {
    let width = (radius * 2 + 1) as usize;
    let plane = width * width;
    if index >= plane * (vertical_range * 2 + 1) as usize {
        return None;
    }

    let dx = (index % width) as i32 - radius;
    let dz = ((index / width) % width) as i32 - radius;
    if dx * dx + dz * dz > radius * radius {
        return None;
    }
    let dy = (index / plane) as i32 - vertical_range;
    Some((center.0 + dx, center.1 + dy, center.2 + dz))
}

fn is_safe_hole(blocks: &HashMap<(i32, i32, i32), ScannedBlock>, x: i32, y: i32, z: i32) -> bool {
    let is = |x, y, z, expected| blocks.get(&(x, y, z)) == Some(&expected);
    if !is(x, y, z, ScannedBlock::Air)
        || !is(x, y + 1, z, ScannedBlock::Air)
        || !matches!(blocks.get(&(x, y - 1, z)), Some(ScannedBlock::SafeMaterial))
    {
        return false;
    }

    [(-1, 0), (1, 0), (0, -1), (0, 1)]
        .into_iter()
        .all(|(dx, dz)| {
            is(x + dx, y, z + dz, ScannedBlock::SafeMaterial)
                && is(x + dx, y + 1, z + dz, ScannedBlock::SafeMaterial)
        })
}

fn gather_entities(
    previous: &[EntityTarget],
    range_sq: f64,
    want_player: bool,
    want_mob: bool,
) -> anyhow::Result<Vec<EntityTarget>> {
    let mc = minecraft();
    let (Some(world), Some(player)) = (mc.world()?, mc.player()?) else {
        return Ok(Vec::new());
    };

    let local_id = player.id()?;
    let player_pos: V3 = player.get_position()?.into();

    let prev_pos: HashMap<i32, V3> = previous.iter().map(|e| (e.id, e.pos)).collect();

    let mut out: Vec<EntityTarget> = Vec::new();
    for entity in world.get_entities()? {
        if let Some(target) = process_entity(
            &entity,
            local_id,
            player_pos,
            range_sq,
            want_player,
            want_mob,
            &prev_pos,
        ) {
            out.push(target);
        }
    }

    Ok(out)
}

#[allow(clippy::too_many_arguments)]
fn process_entity(
    entity: &Entity,
    local_id: i32,
    player_pos: V3,
    range_sq: f64,
    want_player: bool,
    want_mob: bool,
    prev_pos: &HashMap<i32, V3>,
) -> Option<EntityTarget> {
    if let Ok(dist_sq) = entity.distance_to_sqr(player_pos.x, player_pos.y, player_pos.z) {
        if dist_sq > range_sq {
            return None;
        }
    }

    let kind = if want_player && entity.instance_of::<Player>() {
        TargetKind::Player
    } else if want_mob && entity.instance_of::<Mob>() {
        TargetKind::Mob
    } else {
        return None;
    };

    let id = entity.id().ok()?;
    if id == local_id {
        return None;
    }

    let pos: V3 = entity.get_position().ok()?.into();
    let width = entity.bb_width().ok()? as f64;
    let height = entity.bb_height().ok()? as f64;

    let identity_name = read_full_name(entity);
    let name = truncate_name(&identity_name);
    let (health, max_health) = read_health(entity).unwrap_or((0.0, 0.0));

    Some(EntityTarget {
        id,
        kind,
        uuid: if kind == TargetKind::Player {
            entity.uuid().ok()
        } else {
            None
        },
        identity_name,
        prev: prev_pos.get(&id).copied().unwrap_or(pos),
        pos,
        width,
        height,
        name,
        health,
        max_health,
    })
}

fn truncate_name(name: &str) -> String {
    if name.chars().count() > 24 {
        name.chars().take(24).collect()
    } else {
        name.to_owned()
    }
}

fn read_full_name(entity: &Entity) -> String {
    entity
        .get_name()
        .and_then(|component| component.get_string())
        .unwrap_or_default()
}

fn is_friend(entity: &EntityTarget) -> bool {
    match crate::friends::is_friend(entity.uuid.as_deref(), &entity.identity_name) {
        Ok(is_friend) => is_friend,
        Err(error) => {
            log::debug!("friend lookup failed: {error}");
            false
        }
    }
}

fn toggle_crosshair_friend(state: &mut EspState, ctx: &Context) {
    let Some(view) = read_view(state, ctx) else {
        return;
    };
    let targets = match gather_entities(&[], f64::INFINITY, true, false) {
        Ok(targets) => targets,
        Err(error) => {
            log::debug!("middle-click friend lookup failed: {error}");
            return;
        }
    };

    let selected = targets
        .iter()
        .filter_map(|entity| {
            let half = entity.width * 0.5;
            ray_box_distance(
                view.cam,
                view.fwd,
                V3 {
                    x: entity.pos.x - half,
                    y: entity.pos.y,
                    z: entity.pos.z - half,
                },
                V3 {
                    x: entity.pos.x + half,
                    y: entity.pos.y + entity.height,
                    z: entity.pos.z + half,
                },
            )
            .map(|distance| (entity, distance))
        })
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(entity, _)| entity);

    let Some(entity) = selected else {
        return;
    };
    match crate::friends::is_friend(entity.uuid.as_deref(), &entity.identity_name) {
        Ok(true) => {
            match crate::friends::remove_friend(entity.uuid.as_deref(), &entity.identity_name) {
                Ok(true) => log::info!("Removed {} from friends", entity.identity_name),
                Ok(false) => {}
                Err(error) => log::warn!("Could not remove friend: {error}"),
            }
        }
        Ok(false) => {
            match crate::friends::add_friend(entity.uuid.as_deref(), &entity.identity_name) {
                Ok(true) => log::info!("Added {} to friends", entity.identity_name),
                Ok(false) => {}
                Err(error) => log::warn!("Could not add friend: {error}"),
            }
        }
        Err(error) => log::warn!("Could not check friend status: {error}"),
    }
}

fn ray_box_distance(origin: V3, direction: V3, min: V3, max: V3) -> Option<f64> {
    let mut near = 0.0_f64;
    let mut far = f64::INFINITY;
    for (origin, direction, min, max) in [
        (origin.x, direction.x, min.x, max.x),
        (origin.y, direction.y, min.y, max.y),
        (origin.z, direction.z, min.z, max.z),
    ] {
        if direction.abs() < 1e-9 {
            if origin < min || origin > max {
                return None;
            }
            continue;
        }
        let a = (min - origin) / direction;
        let b = (max - origin) / direction;
        near = near.max(a.min(b));
        far = far.min(a.max(b));
        if near > far {
            return None;
        }
    }
    (far >= 0.0).then_some(near)
}

fn read_health(entity: &Entity) -> Option<(f32, f32)> {
    let living = entity.as_living()?;
    Some((living.get_health().ok()?, living.get_max_health().ok()?))
}

fn gather_chests() -> anyhow::Result<Vec<ChestTarget>> {
    let mc = minecraft();
    let (Some(world), Some(player)) = (mc.world()?, mc.player()?) else {
        return Ok(Vec::new());
    };

    let player_pos: V3 = player.get_position()?.into();
    let pcx = (player_pos.x / 16.0).floor() as i32;
    let pcz = (player_pos.z / 16.0).floor() as i32;

    let mut out: Vec<ChestTarget> = Vec::new();
    for cx in (pcx - CHEST_CHUNK_RADIUS)..=(pcx + CHEST_CHUNK_RADIUS) {
        for cz in (pcz - CHEST_CHUNK_RADIUS)..=(pcz + CHEST_CHUNK_RADIUS) {
            scan_chunk(&world, cx, cz, &mut out)?;
        }
    }

    Ok(out)
}

fn scan_chunk(world: &World, cx: i32, cz: i32, out: &mut Vec<ChestTarget>) -> anyhow::Result<()> {
    let Some(chunk) = world.get_chunk(cx, cz)? else {
        return Ok(());
    };

    for block_entity in chunk.get_block_entities()? {
        if !block_entity.is_container() {
            continue;
        }
        if let Ok(pos) = block_entity.get_block_pos() {
            out.push(ChestTarget {
                pos: V3 {
                    x: pos.x() as f64,
                    y: pos.y() as f64,
                    z: pos.z() as f64,
                },
            });
        }
    }
    Ok(())
}

const EDGES: [(usize, usize); 12] = [
    (0, 1),
    (1, 2),
    (2, 3),
    (3, 0),
    (4, 5),
    (5, 6),
    (6, 7),
    (7, 4),
    (0, 4),
    (1, 5),
    (2, 6),
    (3, 7),
];

fn box_corners(min: V3, max: V3) -> [V3; 8] {
    [
        V3 {
            x: min.x,
            y: min.y,
            z: min.z,
        },
        V3 {
            x: max.x,
            y: min.y,
            z: min.z,
        },
        V3 {
            x: max.x,
            y: min.y,
            z: max.z,
        },
        V3 {
            x: min.x,
            y: min.y,
            z: max.z,
        },
        V3 {
            x: min.x,
            y: max.y,
            z: min.z,
        },
        V3 {
            x: max.x,
            y: max.y,
            z: min.z,
        },
        V3 {
            x: max.x,
            y: max.y,
            z: max.z,
        },
        V3 {
            x: min.x,
            y: max.y,
            z: max.z,
        },
    ]
}

fn draw_wire_box(
    painter: &Painter,
    view: &View,
    corners: &[V3; 8],
    color: Color32,
) -> Option<Rect> {
    let projected: [Option<Pos2>; 8] = std::array::from_fn(|i| view.project(corners[i]));
    let stroke = Stroke::new(LINE_WIDTH, color);

    for &(a, b) in &EDGES {
        if let (Some(pa), Some(pb)) = (projected[a], projected[b]) {
            painter.line_segment([pa, pb], stroke);
        }
    }

    let mut bounds: Option<Rect> = None;
    for point in projected.into_iter().flatten() {
        bounds = Some(match bounds {
            Some(rect) => rect.union(Rect::from_min_max(point, point)),
            None => Rect::from_min_max(point, point),
        });
    }
    bounds
}

fn draw_entity(painter: &Painter, view: &View, entity: &EntityTarget, t: f64, cfg: &EspConfig) {
    let icfg = match entity.kind {
        TargetKind::Player => &cfg.player,
        TargetKind::Mob => &cfg.mob,
    };

    if !icfg.enabled {
        return;
    }
    if entity.kind == TargetKind::Player && icfg.ignore_friends && is_friend(entity) {
        return;
    }

    let feet = entity.prev.lerp(entity.pos, t);
    let half = entity.width * 0.5;
    let corners = box_corners(
        V3 {
            x: feet.x - half,
            y: feet.y,
            z: feet.z - half,
        },
        V3 {
            x: feet.x + half,
            y: feet.y + entity.height,
            z: feet.z + half,
        },
    );

    let rect = match draw_wire_box(painter, view, &corners, icfg.color) {
        Some(rect) => rect,
        None => return,
    };

    if icfg.show_health && entity.max_health > 0.0 {
        draw_health_bar(painter, rect, entity.health / entity.max_health);
    }
    if icfg.show_name && !entity.name.is_empty() {
        draw_label(
            painter,
            pos2(rect.center().x, rect.top() - 7.0),
            Align2::CENTER_BOTTOM,
            &entity.name,
            icfg.color,
        );
    }
    if icfg.show_distance {
        let distance = feet.sub(view.cam).length();
        draw_label(
            painter,
            pos2(rect.center().x, rect.bottom() + 3.0),
            Align2::CENTER_TOP,
            &format!("{distance:.0}m"),
            Color32::from_rgb(214, 216, 224),
        );
    }
}

fn draw_tracer(painter: &Painter, view: &View, entity: &EntityTarget, t: f64, cfg: &TracerCfg) {
    let want = match entity.kind {
        TargetKind::Player => cfg.players,
        TargetKind::Mob => cfg.mobs,
    };
    if !want {
        return;
    }

    let feet = entity.prev.lerp(entity.pos, t);
    let center = V3 {
        x: feet.x,
        y: feet.y + entity.height * 0.5,
        z: feet.z,
    };
    if center.sub(view.cam).length() > cfg.range as f64 {
        return;
    }

    let Some(target) = view.project(center) else {
        return;
    };
    let origin = pos2(view.w * 0.5, view.h);
    painter.line_segment([origin, target], Stroke::new(LINE_WIDTH, cfg.color));
}

fn draw_nametag(painter: &Painter, view: &View, entity: &EntityTarget, t: f64, cfg: &NametagCfg) {
    if entity.kind != TargetKind::Player || entity.name.is_empty() {
        return;
    }
    if cfg.ignore_friends && is_friend(entity) {
        return;
    }

    let feet = entity.prev.lerp(entity.pos, t);
    let head = V3 {
        x: feet.x,
        y: feet.y + entity.height + 0.45,
        z: feet.z,
    };
    if head.sub(view.cam).length() > cfg.range as f64 {
        return;
    }
    let Some(anchor) = view.project(head) else {
        return;
    };

    let mut label = entity.name.clone();
    if cfg.show_health && entity.max_health > 0.0 {
        label.push_str(&format!("  {:.0}\u{2764}", entity.health.max(0.0)));
    }
    if cfg.show_distance {
        label.push_str(&format!("  {:.0}m", feet.sub(view.cam).length()));
    }

    let galley = painter.layout_no_wrap(label, FontId::proportional(12.0), cfg.color);
    let pad = vec2(5.0, 2.5);
    let size = galley.size() + pad * 2.0;

    let rect = Rect::from_min_size(pos2(anchor.x - size.x * 0.5, anchor.y - size.y), size);
    painter.rect_filled(rect, Rounding::same(3.0), Color32::from_black_alpha(180));
    painter.galley(rect.min + pad, galley, cfg.color);
}

fn draw_chest(painter: &Painter, view: &View, chest: &ChestTarget, cfg: &EspConfig) {
    let corners = box_corners(
        chest.pos,
        V3 {
            x: chest.pos.x + 1.0,
            y: chest.pos.y + 1.0,
            z: chest.pos.z + 1.0,
        },
    );
    let rect = match draw_wire_box(painter, view, &corners, cfg.chest.color) {
        Some(rect) => rect,
        None => return,
    };

    if cfg.chest.show_distance {
        let center = V3 {
            x: chest.pos.x + 0.5,
            y: chest.pos.y + 0.5,
            z: chest.pos.z + 0.5,
        };
        let distance = center.sub(view.cam).length();
        draw_label(
            painter,
            pos2(rect.center().x, rect.bottom() + 3.0),
            Align2::CENTER_TOP,
            &format!("{distance:.0}m"),
            cfg.chest.color,
        );
    }
}

fn draw_hole(painter: &Painter, view: &View, hole: &HoleTarget, color: Color32) {
    let corners = box_corners(
        hole.pos,
        V3 {
            x: hole.pos.x + 1.0,
            y: hole.pos.y + 2.0,
            z: hole.pos.z + 1.0,
        },
    );
    draw_wire_box(painter, view, &corners, color);
}

fn draw_mineral(painter: &Painter, view: &View, mineral: &MineralTarget, color: Color32) {
    let corners = box_corners(
        mineral.pos,
        V3 {
            x: mineral.pos.x + 1.0,
            y: mineral.pos.y + 1.0,
            z: mineral.pos.z + 1.0,
        },
    );
    let Some(rect) = draw_wire_box(painter, view, &corners, color) else {
        return;
    };
    draw_label(
        painter,
        pos2(rect.center().x, rect.top() - 3.0),
        Align2::CENTER_BOTTOM,
        mineral.name,
        color,
    );
}

fn draw_health_bar(painter: &Painter, rect: Rect, fraction: f32) {
    let fraction = fraction.clamp(0.0, 1.0);
    let top = rect.top() - 4.0;
    let bottom = rect.top() - 1.5;
    let background = Rect::from_min_max(pos2(rect.left(), top), pos2(rect.right(), bottom));
    painter.rect_filled(background, Rounding::ZERO, Color32::from_black_alpha(190));

    let fill_width = rect.width() * fraction;
    let fill = Rect::from_min_max(
        pos2(rect.left(), top),
        pos2(rect.left() + fill_width, bottom),
    );
    painter.rect_filled(fill, Rounding::ZERO, health_color(fraction));
}

fn health_color(fraction: f32) -> Color32 {
    let (r, g) = if fraction < 0.5 {
        (255.0, 255.0 * (fraction * 2.0))
    } else {
        (255.0 * (1.0 - (fraction - 0.5) * 2.0), 255.0)
    };
    Color32::from_rgb(r as u8, g as u8, 60)
}

fn draw_label(painter: &Painter, pos: Pos2, anchor: Align2, text: &str, color: Color32) {
    let font = FontId::proportional(11.0);
    painter.text(
        pos + vec2(0.8, 0.8),
        anchor,
        text,
        font.clone(),
        Color32::from_black_alpha(210),
    );
    painter.text(pos, anchor, text, font, color);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v3(x: f64, y: f64, z: f64) -> V3 {
        V3 { x, y, z }
    }

    #[test]
    fn v3_length_and_dot() {
        let a = v3(3.0, 4.0, 0.0);
        assert_eq!(a.length(), 5.0);
        assert_eq!(a.dot(a), 25.0);
    }

    #[test]
    fn v3_cross_of_x_and_y_is_z() {
        let z = v3(1.0, 0.0, 0.0).cross(v3(0.0, 1.0, 0.0));
        assert!(z.sub(v3(0.0, 0.0, 1.0)).length() < 1e-9);
    }

    #[test]
    fn v3_lerp_finds_the_midpoint() {
        let m = v3(0.0, 0.0, 0.0).lerp(v3(10.0, 20.0, -4.0), 0.5);
        assert_eq!((m.x, m.y, m.z), (5.0, 10.0, -2.0));
    }

    #[test]
    fn a_point_dead_ahead_projects_to_the_screen_centre() {
        let view = build_view(v3(0.0, 0.0, 0.0), 0.0, 0.0, 70.0, 1920.0, 1080.0);
        let projected = view
            .project(v3(0.0, 0.0, 10.0))
            .expect("a point straight ahead must project");
        assert!((projected.x - 960.0).abs() < 1.0);
        assert!((projected.y - 540.0).abs() < 1.0);
    }

    #[test]
    fn a_point_behind_the_camera_does_not_project() {
        let view = build_view(v3(0.0, 0.0, 0.0), 0.0, 0.0, 70.0, 1920.0, 1080.0);
        assert!(view.project(v3(0.0, 0.0, -10.0)).is_none());
    }

    #[test]
    fn center_ray_selects_only_the_nearest_box_in_front() {
        let origin = v3(0.0, 0.0, 0.0);
        let direction = v3(0.0, 0.0, 1.0);
        let near = ray_box_distance(origin, direction, v3(-1.0, -1.0, 4.0), v3(1.0, 2.0, 5.0));
        let far = ray_box_distance(origin, direction, v3(-1.0, -1.0, 8.0), v3(1.0, 2.0, 9.0));
        let behind = ray_box_distance(origin, direction, v3(-1.0, -1.0, -5.0), v3(1.0, 2.0, -4.0));
        assert_eq!(near, Some(4.0));
        assert_eq!(far, Some(8.0));
        assert_eq!(behind, None);
        assert!(near.unwrap() < far.unwrap());
    }

    #[test]
    fn box_corners_span_min_to_max() {
        let corners = box_corners(v3(0.0, 0.0, 0.0), v3(1.0, 2.0, 3.0));
        assert_eq!(corners.len(), 8);
        assert_eq!((corners[0].x, corners[0].y, corners[0].z), (0.0, 0.0, 0.0));
        assert_eq!((corners[6].x, corners[6].y, corners[6].z), (1.0, 2.0, 3.0));
    }

    #[test]
    fn safe_hole_requires_air_interior_and_safe_floor_and_walls() {
        let mut blocks = HashMap::new();
        blocks.insert((0, 0, 0), ScannedBlock::Air);
        blocks.insert((0, 1, 0), ScannedBlock::Air);
        blocks.insert((0, -1, 0), ScannedBlock::SafeMaterial);
        for (dx, dz) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
            blocks.insert((dx, 0, dz), ScannedBlock::SafeMaterial);
            blocks.insert((dx, 1, dz), ScannedBlock::SafeMaterial);
        }

        assert!(is_safe_hole(&blocks, 0, 0, 0));
        blocks.insert((1, 1, 0), ScannedBlock::Other);
        assert!(!is_safe_hole(&blocks, 0, 0, 0));
        blocks.insert((1, 1, 0), ScannedBlock::SafeMaterial);
        blocks.insert((0, -1, 0), ScannedBlock::Other);
        assert!(!is_safe_hole(&blocks, 0, 0, 0));
    }

    #[test]
    fn xray_scan_covers_vertical_layers_and_skips_horizontal_corners() {
        let center = (10, 20, -4);
        assert_eq!(xray_grid_position(center, 1, 1, 3), Some((9, 19, -4)));
        assert_eq!(xray_grid_position(center, 1, 1, 13), Some(center));
        assert_eq!(xray_grid_position(center, 1, 1, 10), Some((10, 20, -5)));
        assert_eq!(xray_grid_position(center, 1, 1, 19), Some((10, 21, -5)));
        assert_eq!(xray_grid_position(center, 1, 1, 2), None);
        assert_eq!(xray_grid_position(center, 1, 1, 27), None);
    }
}
