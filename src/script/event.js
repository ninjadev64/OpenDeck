const { pluginManager } = require("./plugins");
const { keys } = require("./shared");

class EventHandler {
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
}

const eventHandler = new EventHandler();
module.exports = { eventHandler };