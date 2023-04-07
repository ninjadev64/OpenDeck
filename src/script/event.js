const { shell } = require("electron");
const { pluginManager } = require("./plugins");
const { keys, sliders } = require("./shared");

const log = require("electron-log");

class EventHandler {
	// Outbound events

	keyDown(key) {
		let action = keys[key];
		if (action == undefined) return;
		pluginManager.sendEvent(action.plugin, {
			event: "keyDown",
			action: action.uuid,
			context: key,
			device: 0,
			payload: {
				settings: {},
				coordinates: {
					row: Math.floor(key / 3) + 1,
					column: key % 3
				},
				isInMultiAction: false
			}
		});
	}

	keyUp(key) {
		let action = keys[key];
		if (action == undefined) return;
		pluginManager.sendEvent(action.plugin, {
			event: "keyUp",
			action: action.uuid,
			context: key,
			device: 0,
			payload: {
				settings: {},
				coordinates: {
					row: Math.floor(key / 3) + 1,
					column: key % 3
				},
				isInMultiAction: false
			}
		});
	}

	dialRotate(slider, value) {
		let action = sliders[slider];
		if (action == undefined) return;
		pluginManager.sendEvent(action.plugin, {
			event: "dialRotate",
			action: action.uuid,
			context: `s${slider}`,
			device: 0,
			payload: {
				settings: {},
				coordinates: {
					column: slider,
					row: 0
				},
				ticks: value,
				pressed: false
			}
		});
	}

	willAppear(key) {
		let action = keys[key];
		if (action == undefined) return;
		pluginManager.sendEvent(action.plugin, {
			event: "willAppear",
			action: action.uuid,
			context: key,
			device: 0,
			payload: {
				controller: "Keypad",
				settings: {},
				coordinates: {
					row: Math.floor(key / 3) + 1,
					column: key % 3
				},
				isInMultiAction: false
			}
		});
	}

	willDisappear(key) {
		let action = keys[key];
		if (action == undefined) return;
		pluginManager.sendEvent(action.plugin, {
			event: "willDisappear",
			action: action.uuid,
			context: key,
			device: 0,
			payload: {
				controller: "Keypad",
				settings: {},
				coordinates: {
					row: Math.floor(key / 3) + 1,
					column: key % 3
				},
				isInMultiAction: false
			}
		});
	}

	deviceDidConnect() {
		pluginManager.sendGlobalEvent({
			event: "deviceDidConnect",
			device: 0,
			deviceInfo: {
				name: "OceanDeck",
				type: 7,
				size: {
					rows: 3,
					columns: 3
				}
			}
		});
	}

	deviceDidDisconnect() {
		pluginManager.sendGlobalEvent({
			event: "deviceDidDisconnect",
			device: 0
		});
	}

	propertyInspectorDidAppear(key) {
		let action = keys[key];
		pluginManager.sendEvent(action.plugin, {
			event: "propertyInspectorDidAppear",
			action: action.uuid,
			context: key,
			device: 0
		});
	}

	propertyInspectorDidDisappear(key) {
		let action = keys[key];
		pluginManager.sendEvent(action.plugin, {
			event: "propertyInspectorDidDisappear",
			action: action.uuid,
			context: key,
			device: 0
		});
	}

	// Inbound events

	openUrl(data) {
		shell.openExternal(data.payload.url);
	}

	logMessage(data) {
		log.debug(data.payload.message);
	}
}

const eventHandler = new EventHandler();
module.exports = { eventHandler };