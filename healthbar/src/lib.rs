use core::{
    arch::asm,
    cmp::{max, min},
    ffi::c_void,
};

use std::any::type_name_of_val;

use iced_x86::code_asm::*;
use windows::core::{
    PCSTR,
    s,
};
use windows::Win32::{
    Foundation::HINSTANCE,
    Foundation::HWND,
    System::SystemServices::DLL_PROCESS_ATTACH,
    UI::WindowsAndMessaging::MessageBoxA,
    UI::WindowsAndMessaging::MB_OK,
};

mod config;
mod memory;

use config::Config;
use memory::inject;

#[derive(PartialEq, Clone, Copy)]
enum HealthBarVisibility {
    Never,
    Damaged,
    Always,
}

#[derive(PartialEq, Clone, Copy)]
enum HealthBarColor {
    Default,
    Teamcolor,
}

static mut HB_COLOR: HealthBarColor = HealthBarColor::Default;
static mut PLANT_HB_VISIBILITY: HealthBarVisibility = HealthBarVisibility::Damaged;
static mut ZOMBIE_HB_VISIBILITY: HealthBarVisibility = HealthBarVisibility::Never;
static mut WAVE_METER_VISIBILITY: bool = true;

// FIXME: Crash after sometime
#[inline]
fn fill_rect(this: &mut pvz::Graphics, color: pvz::Color, x: i32, y: i32, width: i32, height: i32) {
    let old_color = this.state.mColor;
    this.state.mColor = color;
    unsafe {
        asm!(
            "pushad",
            "push {height}",
            "push {width}",
            "push {y}",
            "push {x}",
            "mov ecx, {this}",
            "call {func}",
            "popad",
            x = in(reg) x,
            y = in(reg) y,
            width = in(reg) width,
            height = in(reg) height,
            this = in(reg) this,
            func = in(reg) 0x59eb80,
        );
    };
    this.state.mColor = old_color;
}

#[inline]
fn draw_health_bar(
    graphics: &mut pvz::Graphics,
    hb_color: pvz::Color,
    x: i32,
    y: i32,
    width: i32,
    max_width: i32,
) {
    let black = pvz::Color::new(0x000000FF);
    let dark_gray = pvz::Color::new(0x303030FF);
    fill_rect(graphics, black, x - 1, y - 1, max_width + 2, 5);
    fill_rect(graphics, hb_color, x, y, width, 3);
    fill_rect(graphics, dark_gray, x + width, y, max_width - width, 3);
}

#[inline]
fn draw_vert_hb(
    graphics: &mut pvz::Graphics,
    hb_color: pvz::Color,
    x: i32,
    y: i32,
    height: i32,
    max_height: i32,
) {
    let black = pvz::Color::new(0x000000FF);
    let dark_gray = pvz::Color::new(0x303030FF);
    fill_rect(graphics, black, x - 1, y - 1, 7, max_height + 2);
    fill_rect(graphics, hb_color, x, y + max_height - height, 5, height);
    fill_rect(graphics, dark_gray, x, y, 5, max_height - height);
}

fn draw_plant_health_bar(graphics: &mut pvz::Graphics, plant: &pvz::Plant) {
    if plant.mDead || plant.mSquished {
        return;
    }

    if let HealthBarVisibility::Damaged = unsafe { PLANT_HB_VISIBILITY } {
        if plant.mPlantHealth == plant.mPlantMaxHealth {
            return;
        }
    }

    let hb_color = if let HealthBarColor::Default = unsafe { HB_COLOR } {
        calc_health_color(plant.mPlantHealth as f32 / plant.mPlantMaxHealth as f32)
    } else {
        pvz::Color::new(0x008FC7FF)
    };

    let max_hb = if let pvz::SeedType::Cobcannon = plant.mSeedType {
        140
    } else {
        60
    };
    let hb_length = max_hb * plant.mPlantHealth / plant.mPlantMaxHealth;

    let extra_y = match plant.mSeedType {
        pvz::SeedType::Spikeweed
        | pvz::SeedType::Spikerock
        | pvz::SeedType::Flowerpot
        | pvz::SeedType::Lilypad => 50,
        pvz::SeedType::Tanglekelp | pvz::SeedType::Pumpkinshell => 35,
        _ => 0,
    };

    draw_health_bar(
        graphics,
        hb_color,
        plant.base.mX + 10,
        plant.base.mY + extra_y,
        hb_length,
        max_hb,
    );
}

#[inline]
fn calc_health_color(hp_fraction: f32) -> pvz::Color {
    pvz::Color {
        mRed: (255.0 * (1.0 - hp_fraction).sqrt()) as i32,
        mGreen: (255.0 * hp_fraction.sqrt()) as i32,
        mBlue: 0,
        mAlpha: 255,
    }
}

fn draw_zombie_health_bar(graphics: &mut pvz::Graphics, zombie: &pvz::Zombie) {
    if zombie.mDead || zombie.mBlowingAway || zombie.mFlatTires || zombie.mBodyHealth == 0 {
        return;
    }
    if let pvz::ZombieType::Boss = zombie.mZombieType {
        return;
    }
    if let HealthBarVisibility::Damaged = unsafe { ZOMBIE_HB_VISIBILITY } {
        if zombie.mBodyHealth == zombie.mBodyMaxHealth
            && zombie.mHelmHealth == zombie.mHelmMaxHealth
            && zombie.mShieldHealth == zombie.mShieldMaxHealth
        {
            return;
        }
    }

    let max_body_health = zombie.mBodyMaxHealth + zombie.mHelmMaxHealth;
    let body_health = zombie.mBodyHealth + zombie.mHelmHealth;
    let max_health = max_body_health + zombie.mShieldMaxHealth;
    let hb_length = zombie.mZombieRect.mWidth + 20;
    let shield_hb_length: i32 = hb_length * zombie.mShieldMaxHealth / max_health - 1;
    let body_hb_length: i32 = hb_length - shield_hb_length - 1;

    let extra_y = match zombie.mZombieType {
        pvz::ZombieType::Imp => 35,
        _ => 0,
    };

    let hb_color = if let HealthBarColor::Default = unsafe { HB_COLOR } {
        calc_health_color(body_health as f32 / max_body_health as f32)
    } else {
        pvz::Color::new(0xA10B0BFF)
    };

    let body_hb_current = body_health * body_hb_length / max_body_health;
    draw_health_bar(
        graphics,
        hb_color,
        zombie.base.mX + zombie.mZombieRect.mX - 10,
        zombie.base.mY + zombie.mZombieRect.mY + extra_y - zombie.mAltitude as i32,
        body_hb_current,
        body_hb_length,
    );
    if zombie.mShieldMaxHealth == 0 {
        return;
    }
    let shield_hb_current = zombie.mShieldHealth * shield_hb_length / zombie.mShieldMaxHealth;
    draw_health_bar(
        graphics,
        pvz::Color::new(0xFFFFFFFF),
        zombie.base.mX + zombie.mZombieRect.mX - 9 + body_hb_length,
        zombie.base.mY + zombie.mZombieRect.mY + extra_y - zombie.mAltitude as i32,
        shield_hb_current,
        shield_hb_length,
    );
}

unsafe fn draw_wave_meter(board: &mut pvz::Board, graphics: &mut pvz::Graphics) {
    // match &(*board.mApp).mGameMode {
    //     pvz::GameMode::ChallengeZombiquarium
    //     | pvz::GameMode::ChallengeSquirrel
    //     | pvz::GameMode::TreeOfWisdom
    //     | pvz::GameMode::ChallengeZenGarden
    //     | pvz::GameMode::ChallengeFinalBoss => return,
    //     pvz::GameMode::Adventure => {
    //         if board.mLevel == 50 {
    //             return;
    //         }
    //     }
    //     _ => {}
    // }

//     let zombie_head_image =
//         (*(pvz::TodParticleDefinition::get(pvz::ParticleEffect::ZombieHead).mEmitterDefs)).mImage;

//     let old_scale_x = graphics.state.mScaleX;
//     let old_scale_y = graphics.state.mScaleY;
//     graphics.state.mScaleX = 0.5 * old_scale_x;
//     graphics.state.mScaleY = 0.5 * old_scale_y;
//     graphics.DrawImage(
//         zombie_head_image,
//         1700 + board.base.base.mX,
//         700 + board.base.base.mY,
//     );
//     graphics.state.mScaleX = old_scale_x;
//     graphics.state.mScaleY = old_scale_y;

    //MessageBoxA(None, s!("Test"), s!("Debug"), MB_OK);

    let meter_height: i32;
    let meter_color: pvz::Color;

    if board.mCurrentWave == 0 {
        MessageBoxA(None, s!("Wave 0"), s!("Debug"), MB_OK);
        meter_height = 100 - 100 * board.mZombieCountDown / board.mZombieCountDownStart;
        meter_color = pvz::Color::new(0xFFFFFFFF);
    } else if board.mHugeWaveCountDown > 0 {
        //MessageBoxA(None, s!("Wave CountDown > 0"), s!("Debug"), MB_OK);
        meter_height = 100 - 100 * board.mHugeWaveCountDown / 750;
        meter_color = pvz::Color::new(0xA10B0BFF);
    } else if board.mZombieCountDown <= 200 {
        meter_height = 100 - 100 * board.mZombieCountDown / 200;
        meter_color = pvz::Color::new(0xFFFFFFFF);
    } else {
        meter_height = 100
            * (board.TotalZombiesHealthInWave(board.mCurrentWave - 1)
                - board.mZombieHealthToNextWave)
            / (board.mZombieHealthWaveStart - board.mZombieHealthToNextWave);
        meter_color = pvz::Color::new(0xFF7F0EFF);
    }

    //MessageBoxA(None, s!("Try to draw"), s!("Debug"), MB_OK);

    draw_vert_hb(
        graphics,
        meter_color,
        863,
        250,
        min(max(meter_height, 0), 100),
        100,
    );

    //MessageBoxA(None, s!("Done drawing"), s!("Debug"), MB_OK);
}

#[no_mangle]
unsafe extern "cdecl" fn draw_all_health_bar(board: *mut pvz::Board, graphics: *mut pvz::Graphics) {
    let board = &mut *board;
    let graphics = &mut *graphics;

    // if !matches!(PLANT_HB_VISIBILITY, HealthBarVisibility::Never) {
    //     for plant in &board.mPlants {
    //         draw_plant_health_bar(graphics, plant);
    //     }
    // }

    // if !matches!(ZOMBIE_HB_VISIBILITY, HealthBarVisibility::Never) {
    //     for zombie in &board.mZombies {
    //         draw_zombie_health_bar(graphics, zombie);
    //     }
    // }

    if WAVE_METER_VISIBILITY {
        //MessageBoxA(None, s!("Try to draw"), s!("Debug"), MB_OK);
        draw_wave_meter(board, graphics);
    }
}

unsafe fn onboarddraw() -> Result<(), Box<IcedError>> {
    let mut code = CodeAssembler::new(32)?;
    code.pushad()?;
    code.push(dword_ptr(esp + 0x28))?;
    code.push(dword_ptr(esp + 0x28))?;
    code.call(draw_all_health_bar as *const () as u64)?;
    code.add(esp, 8)?;
    code.popad()?;
    code.ret_1(8)?;

    inject(0x41a80c, code);

    Ok(())
}

#[no_mangle] // call it "DllMain" in the compiled DLL
#[allow(unused_variables)]
pub extern "stdcall" fn DllMain(
    hinst_dll: HINSTANCE,
    fdw_reason: u32,
    lp_reserved: *mut c_void,
) -> i32 {
    match fdw_reason {
        DLL_PROCESS_ATTACH => {
            let config = Config::get_config();

            if let Some(color) = config.color {
                unsafe {
                    if color == "Default" {
                        HB_COLOR = HealthBarColor::Default;
                    } else if color == "Teamcolor" {
                        HB_COLOR = HealthBarColor::Teamcolor;
                    }
                }
            }

            if let Some(visibility) = config.plant_hb_visibility {
                unsafe {
                    if visibility == "Never" {
                        PLANT_HB_VISIBILITY = HealthBarVisibility::Never;
                    } else if visibility == "Damaged" {
                        PLANT_HB_VISIBILITY = HealthBarVisibility::Damaged;
                    } else if visibility == "Always" {
                        PLANT_HB_VISIBILITY = HealthBarVisibility::Always;
                    }
                }
            }

            if let Some(visibility) = config.zombie_hb_visibility {
                unsafe {
                    if visibility == "Never" {
                        ZOMBIE_HB_VISIBILITY = HealthBarVisibility::Never;
                    } else if visibility == "Damaged" {
                        ZOMBIE_HB_VISIBILITY = HealthBarVisibility::Damaged;
                    } else if visibility == "Always" {
                        ZOMBIE_HB_VISIBILITY = HealthBarVisibility::Always;
                    }
                }
            }

            unsafe {
                WAVE_METER_VISIBILITY = config.wave_meter_visibility.unwrap_or(true);

                onboarddraw().unwrap();
            };
        }
        _ => {}
    }
    true as i32
}
