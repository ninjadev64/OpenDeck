use std::sync::LazyLock;
use std::sync::atomic::{AtomicBool, AtomicU16, Ordering};
use std::time::{Duration, Instant};

use dashmap::DashMap;

static SLEEP_TIMEOUT_MINUTES: AtomicU16 = AtomicU16::new(0);
static LAST_ACTIVITY: LazyLock<DashMap<String, Instant>> = LazyLock::new(DashMap::new);

static SLEEP_WHEN_COMPUTER_LOCKED: AtomicBool = AtomicBool::new(false);
static COMPUTER_LOCKED: AtomicBool = AtomicBool::new(false);
static CLEAR_DISPLAY_ON_SLEEP: AtomicBool = AtomicBool::new(false);

static SLEEPING_DEVICES: LazyLock<DashMap<String, ()>> = LazyLock::new(DashMap::new);

pub fn is_device_sleeping(device: &str) -> bool {
	SLEEPING_DEVICES.contains_key(device)
}

pub fn should_suppress_updates(device: &str) -> bool {
	CLEAR_DISPLAY_ON_SLEEP.load(Ordering::Relaxed) && is_device_sleeping(device)
}

pub fn init_device_sleep() {
	SLEEP_TIMEOUT_MINUTES.store(crate::store::get_settings().value.sleep_timeout_minutes, Ordering::Relaxed);
	SLEEP_WHEN_COMPUTER_LOCKED.store(crate::store::get_settings().value.sleep_when_computer_locked, Ordering::Relaxed);
	CLEAR_DISPLAY_ON_SLEEP.store(crate::store::get_settings().value.clear_display_on_sleep, Ordering::Relaxed);

	tokio::spawn(async {
		loop {
			if let Err(error) = sleep_idle_devices().await {
				log::warn!("Failed to update sleeping devices: {error}");
			}
			tokio::time::sleep(Duration::from_secs(2)).await;
		}
	});
}

pub async fn update_sleep_timeout_minutes(minutes: u16) -> Result<(), anyhow::Error> {
	SLEEP_TIMEOUT_MINUTES.store(minutes, Ordering::Relaxed);

	if minutes == 0 && !(SLEEP_WHEN_COMPUTER_LOCKED.load(Ordering::Relaxed) && COMPUTER_LOCKED.load(Ordering::Relaxed)) {
		for device in SLEEPING_DEVICES.iter().map(|entry| entry.key().clone()).collect::<Vec<_>>() {
			wake_device(&device).await?;
		}
	}

	Ok(())
}

pub async fn note_activity(device: &str) -> Result<bool, anyhow::Error> {
	if SLEEP_WHEN_COMPUTER_LOCKED.load(Ordering::Relaxed) && COMPUTER_LOCKED.load(Ordering::Relaxed) {
		return Ok(true);
	}

	LAST_ACTIVITY.insert(device.to_owned(), Instant::now());
	wake_device(device).await
}

pub fn deregister_device(device: &str) {
	LAST_ACTIVITY.remove(device);
	SLEEPING_DEVICES.remove(device);
}

pub async fn sleep_device(device: String) -> Result<(), anyhow::Error> {
	crate::events::outbound::devices::set_device_brightness(&device, 0).await?;
	if CLEAR_DISPLAY_ON_SLEEP.load(Ordering::Relaxed) {
		crate::events::outbound::devices::clear_on_sleep(&device).await?;
	}
	SLEEPING_DEVICES.insert(device, ());
	Ok(())
}

async fn sleep_idle_devices() -> Result<(), anyhow::Error> {
	let timeout = SLEEP_TIMEOUT_MINUTES.load(Ordering::Relaxed);
	if timeout == 0 {
		return Ok(());
	}

	let idle_after = Duration::from_secs(timeout as u64 * 60);
	let now = Instant::now();
	let device_ids = LAST_ACTIVITY.iter().map(|entry| entry.key().clone()).collect::<Vec<_>>();

	for device in device_ids {
		let Some(last_activity) = LAST_ACTIVITY.get(&device).map(|entry| *entry.value()) else { continue };
		if now.duration_since(last_activity) < idle_after || is_device_sleeping(&device) {
			continue;
		}

		sleep_device(device).await?;
	}

	Ok(())
}

pub async fn wake_device(device: &str) -> Result<bool, anyhow::Error> {
	if SLEEPING_DEVICES.remove(device).is_some() {
		let brightness = crate::store::get_settings().value.brightness;
		crate::events::outbound::devices::set_device_brightness(device, brightness).await?;
		if CLEAR_DISPLAY_ON_SLEEP.load(Ordering::Relaxed) {
			crate::events::frontend::profiles::rerender_images(crate::APP_HANDLE.get().unwrap()).await?;
		}
		return Ok(true);
	}

	Ok(false)
}

pub async fn update_clear_display_on_sleep(enabled: bool) -> Result<(), anyhow::Error> {
	let changed = CLEAR_DISPLAY_ON_SLEEP.swap(enabled, Ordering::Relaxed) != enabled;
	if !changed {
		return Ok(());
	}

	if enabled {
		for device in SLEEPING_DEVICES.iter().map(|entry| entry.key().clone()).collect::<Vec<_>>() {
			crate::events::outbound::devices::clear_on_sleep(&device).await?;
		}
	} else if !SLEEPING_DEVICES.is_empty() {
		crate::events::frontend::profiles::rerender_images(crate::APP_HANDLE.get().unwrap()).await?;
	}

	Ok(())
}

pub async fn update_sleep_when_computer_locked(enabled: bool) -> Result<(), anyhow::Error> {
	SLEEP_WHEN_COMPUTER_LOCKED.store(enabled, Ordering::Relaxed);
	if enabled && COMPUTER_LOCKED.load(Ordering::Relaxed) {
		sleep_for_computer_lock().await?;
	} else if !enabled {
		wake_from_computer_lock().await?;
	}
	Ok(())
}

pub async fn sleep_for_computer_lock() -> Result<(), anyhow::Error> {
	COMPUTER_LOCKED.store(true, Ordering::Relaxed);
	if !SLEEP_WHEN_COMPUTER_LOCKED.load(Ordering::Relaxed) {
		return Ok(());
	}
	let device_ids = crate::shared::DEVICES.iter().map(|entry| entry.id.clone()).collect::<Vec<_>>();
	for device in device_ids {
		sleep_device(device).await?;
	}
	Ok(())
}

pub async fn wake_from_computer_lock() -> Result<(), anyhow::Error> {
	COMPUTER_LOCKED.store(false, Ordering::Relaxed);
	let device_ids = SLEEPING_DEVICES.iter().map(|entry| entry.key().clone()).collect::<Vec<_>>();
	for device in device_ids {
		LAST_ACTIVITY.insert(device.to_owned(), Instant::now());
		wake_device(&device).await?;
	}
	Ok(())
}

pub async fn apply_initial_device_sleep(device: &str) -> Result<(), anyhow::Error> {
	LAST_ACTIVITY.insert(device.to_owned(), Instant::now());
	if SLEEP_WHEN_COMPUTER_LOCKED.load(Ordering::Relaxed) && COMPUTER_LOCKED.load(Ordering::Relaxed) {
		sleep_device(device.to_owned()).await?;
	}
	Ok(())
}
