const { shell } = require("electron");
const { pluginManager } = require("./plugins");
const { propertyInspectorManager } = require("./propertyinspector");
const { getProfile, updateProfile, parseContext, getInstanceByContext, getCoordinatesByContext } = require("./shared");

const log = require("electron-log");
const { getMainWindow } = require("./main");
const store = require("./store");

class EventHandler {
	updateState(instance) {
		let context = parseContext(instance.context);
		getProfile()[context.type][context.position][context.index] = instance;
		updateProfile();
		getMainWindow().webContents.send("updateState", instance);
	}

	// Outbound events

	keyDown(key) {
		let instance = getProfile().key[key][0];
		if (instance == undefined) return;
		pluginManager.sendEvent(instance.action.plugin, {
			event: "keyDown",
			action: instance.action.uuid,
			context: instance.context,
			device: 0,
			payload: {
				settings: instance.settings,
				coordinates: getCoordinatesByContext(instance.context),
				isInMultiAction: false
			}
		});
	}

	keyUp(key) {
		let instance = getProfile().key[key][0];
		if (instance == undefined) return;
		pluginManager.sendEvent(instance.action.plugin, {
			event: "keyUp",
			action: instance.action.uuid,
			context: instance.context,
			device: 0,
			payload: {
				settings: instance.settings,
				coordinates: getCoordinatesByContext(instance.context),
				isInMultiAction: false
			}
		});
		instance.state += 1;
		if (instance.state >= instance.states.length) {
			instance.state = 0;
		}
		this.updateState(instance);
	}

	dialRotate(slider, value) {
		let instance = getProfile().slider[slider][0];
		if (instance == undefined) return;
		pluginManager.sendEvent(instance.action.plugin, {
			event: "dialRotate",
			action: instance.action.uuid,
			context: instance.context,
			device: 0,
			payload: {
				settings: instance.settings,
				coordinates: getCoordinatesByContext(instance.context),
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
				controller: instance.type == "slider" ? "Encoder" : "Keypad",
				settings: instance.settings,
				coordinates: getCoordinatesByContext(instance.context),
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
				controller: instance.type == "slider" ? "Encoder" : "Keypad",
				settings: instance.settings,
				coordinates: getCoordinatesByContext(instance.context),
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
				settings: instance.settings,
				coordinates: getCoordinatesByContext(instance.context),
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
				settings: store.get("pluginSettings." + plugin.uuid.replaceAll(".", "¬")) ?? {},
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
		instance.settings = payload;
		this.updateState(instance);
		this.didReceiveSettings(instance, !fromPropertyInspector);
	}

	getSettings({ context }, fromPropertyInspector) {
		this.didReceiveSettings(getInstanceByContext(context), fromPropertyInspector);
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

	setTitle({ context, payload: { title, state } }) {
		let instance = getInstanceByContext(context);
		if (state) instance.states[state].title = title;
		else instance.states.forEach((state) => state.title = title);
		this.updateState(instance);
	}

	setImage({ context, payload: { image, state } }) {
		let instance = getInstanceByContext(context);
		if (state) instance.states[state].image = image;
		else instance.states.forEach((state) => state.image = image);
		this.updateState(instance);
	}

	setState({ context, payload: { state } }) {
		let instance = getInstanceByContext(context);
		instance.state = state;
		this.updateState(instance);
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