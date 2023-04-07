const { shell } = require("electron");
const { pluginManager } = require("./plugins");
const { keys, sliders } = require("./shared");

const log = require("electron-log");

class EventHandler {
	// Outbound events

	keyDown(key) {
		let instance = keys[key];
		if (instance == undefined) return;
		pluginManager.sendEvent(instance.action.plugin, {
			event: "keyDown",
			action: instance.action.uuid,
			context: key,
			device: 0,
			payload: {
				settings: {},
				coordinates: {
					row: Math.floor((key - 1) / 3),
					column: (key - 1) % 3
				},
				isInMultiAction: false
			}
		});
	}

	keyUp(key) {
		let instance = keys[key];
		if (instance == undefined) return;
		pluginManager.sendEvent(instance.action.plugin, {
			event: "keyUp",
			action: instance.action.uuid,
			context: key,
			device: 0,
			payload: {
				settings: {},
				coordinates: {
					row: Math.floor((key - 1) / 3),
					column: (key - 1) % 3
				},
				isInMultiAction: false
			}
		});
	}

	dialRotate(slider, value) {
		let instance = sliders[slider];
		if (instance == undefined) return;
		pluginManager.sendEvent(instance.action.plugin, {
			event: "dialRotate",
			action: instance.action.uuid,
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

	willAppear(instance) {
		pluginManager.sendEvent(instance.action.plugin, {
			event: "willAppear",
			action: instance.action.uuid,
			context: instance.context,
			device: 0,
			payload: {
				controller: "Keypad",
				settings: {},
				coordinates: {
					row: Math.floor((instance.index - 1) / 3),
					column: (instance.index - 1) % 3
				},
				isInMultiAction: false
			}
		});
	}

	willDisappear(instance) {
		pluginManager.sendEvent(instance.action.plugin, {
			event: "willDisappear",
			action: instance.action.uuid,
			context: instance.context,
			device: 0,
			payload: {
				controller: "Keypad",
				settings: {},
				coordinates: {
					row: Math.floor((instance.index - 1) / 3),
					column: (instance.index - 1) % 3
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

	propertyInspectorDidAppear(instance) {
		pluginManager.sendEvent(instance.action.plugin, {
			event: "propertyInspectorDidAppear",
			action: instance.action.uuid,
			context: instance.context,
			device: 0
		});
	}

	propertyInspectorDidDisappear(instance) {
		pluginManager.sendEvent(instance.action.plugin, {
			event: "propertyInspectorDidDisappear",
			action: instance.action.uuid,
			context: instance.context,
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