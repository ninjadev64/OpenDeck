const { shell } = require("electron");
const { pluginManager } = require("./plugins");
const { propertyInspectorManager } = require("./propertyinspector");
const { keys, sliders, getInstanceByContext } = require("./shared");

const log = require("electron-log");
const { getMainWindow } = require("./main");
const store = require("./store");

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

	didReceiveSettings(instance, propertyInspector) {
		let data = {
			event: "didReceiveSettings",
			action: instance.action.uuid,
			context: instance.context,
			device: 0,
			payload: {
				settings: store.get("actionSettings." + instance.context),
				coordinates: {
					row: Math.floor((instance.index - 1) / 3),
					column: (instance.index - 1) % 3
				},
				isInMultiAction: false
			}
		}
		if (propertyInspector) {
			propertyInspectorManager.sendEvent(instance.context, data);
		} else {
			pluginManager.sendEvent(instance.action.plugin, data);
		}
	}

	didReceiveGlobalSettings(instance, propertyInspector) {
		let data = {
			event: "didReceiveGlobalSettings",
			payload: {
				settings: store.get("pluginSettings." + plugin.uuid.replaceAll(".", "¬")),
			}
		}
		if (propertyInspector) {
			propertyInspectorManager.sendEvent(instance.context, data);
		} else {
			pluginManager.sendEvent(instance.action.plugin, data);
		}
	}

	// Inbound events

	setSettings({ context, payload }, fromPropertyInspector) {
		let instance = getInstanceByContext(context);
		store.set("actionSettings." + instance.context, payload);
		this.didReceiveSettings(instance, !fromPropertyInspector);
	}

	getSettings({ context }, fromPropertyInspector) {
		this.didReceiveSettings(getInstanceByContext(context), fromPropertyInspector)
	}

	setGlobalSettings({ context, payload }, fromPropertyInspector) {
		let plugin = fromPropertyInspector ? propertyInspectorManager.all[context].action.plugin : pluginManager.plugins[context];
		store.set("pluginSettings." + plugin.uuid.replaceAll(".", "¬"), payload);
		this.didReceiveGlobalSettings(plugin, !fromPropertyInspector);
	}

	getGlobalSettings({ context }, fromPropertyInspector) {
		let plugin = fromPropertyInspector ? propertyInspectorManager.all[context].action.plugin : pluginManager.plugins[context];
		this.didReceiveGlobalSettings(plugin, fromPropertyInspector)
	}

	openUrl({ payload: { url } }) {
		shell.openExternal(url);
	}

	logMessage({ payload: { message } }) {
		log.debug(message);
	}

	showAlert({ context }) {
		getMainWindow().webContents.send("showAlert", context);
	}

	showOk({ context }) {
		getMainWindow().webContents.send("showOk", context);
	}
}

const eventHandler = new EventHandler();
module.exports = { eventHandler };