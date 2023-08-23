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
		getProfile(context.device)[context.type][context.position][context.index] = instance;
		updateProfile(context.device);
		getMainWindow().webContents.send("updateState", instance.context, instance);

		const { deviceManager } = require("./devices");
		deviceManager.devices[context.device].setImage(instance.states[instance.state].image);
	}

	// Outbound events
	// Reference: https://docs.elgato.com/sdk/plugins/events-received

	keyDown(device, key) {
		let instance = getProfile(device).key[key][0];
		if (instance == undefined) return;
		pluginManager.sendEvent(instance.action.plugin, {
			event: "keyDown",
			action: instance.action.uuid,
			context: instance.context,
			device: instance.device,
			payload: {
				settings: instance.settings,
				coordinates: getCoordinatesByContext(instance.context),
				isInMultiAction: false
			}
		});
	}

	keyUp(device, key) {
		let instance = getProfile(device).key[key][0];
		if (instance == undefined) return;
		pluginManager.sendEvent(instance.action.plugin, {
			event: "keyUp",
			action: instance.action.uuid,
			context: instance.context,
			device: instance.device,
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

	dialRotate(device, slider, value) {
		let instance = getProfile(device).slider[slider][0];
		if (instance == undefined) return;
		pluginManager.sendEvent(instance.action.plugin, {
			event: "dialRotate",
			action: instance.action.uuid,
			context: instance.context,
			device: instance.device,
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
			device: instance.device,
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
			device: instance.device,
			payload: {
				controller: instance.type == "slider" ? "Encoder" : "Keypad",
				settings: instance.settings,
				coordinates: getCoordinatesByContext(instance.context),
				isInMultiAction: false
			}
		});
	}

	deviceDidConnect(id, device) {
		pluginManager.sendGlobalEvent({
			event: "deviceDidConnect",
			device: id,
			deviceInfo: {
				name: device.name,
				type: device.type,
				size: {
					rows: device.rows,
					columns: device.columns
				}
			}
		});
	}

	deviceDidDisconnect(id) {
		pluginManager.sendGlobalEvent({
			event: "deviceDidDisconnect",
			device: id
		});
	}

	applicationDidLaunch(application, plugin) {
		pluginManager.sendEvent(plugin, {
			event: "applicationDidLaunch",
			payload: {
				application
			}
		});
	}

	applicationDidTerminate(application, plugin) {
		pluginManager.sendEvent(plugin, {
			event: "applicationDidTerminate",
			payload: {
				application
			}
		});
	}

	propertyInspectorDidAppear(instance) {
		pluginManager.sendEvent(instance.action.plugin, {
			event: "propertyInspectorDidAppear",
			action: instance.action.uuid,
			context: instance.context,
			device: instance.device
		});
	}

	propertyInspectorDidDisappear(instance) {
		pluginManager.sendEvent(instance.action.plugin, {
			event: "propertyInspectorDidDisappear",
			action: instance.action.uuid,
			context: instance.context,
			device: instance.device
		});
	}

	didReceiveSettings(instance, propertyInspector) {
		let data = {
			event: "didReceiveSettings",
			action: instance.action.uuid,
			context: instance.context,
			device: instance.device,
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

	didReceiveGlobalSettings(plugin, propertyInspector) {
		let data = {
			event: "didReceiveGlobalSettings",
			payload: {
				settings: store.get("pluginSettings." + plugin.replaceAll(".", "¬")) ?? {},
			}
		}
		if (propertyInspector) {
			Object.values(propertyInspectorManager.all).forEach((propertyInspector) => {
				if (propertyInspector.action.plugin == plugin) {
					propertyInspectorManager.sendEvent(propertyInspector.context, data);
				}
			});
		} else {
			pluginManager.sendEvent(plugin, data);
		}
	}

	// Inbound events
	// Reference: https://docs.elgato.com/sdk/plugins/events-sent

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
		let plugin = fromPropertyInspector ? propertyInspectorManager.all[context].action.plugin : pluginManager.plugins[context].uuid;
		store.set("pluginSettings." + plugin.replaceAll(".", "¬"), payload);
		this.didReceiveGlobalSettings(plugin, !fromPropertyInspector);
	}

	getGlobalSettings({ context }, fromPropertyInspector) {
		let plugin = fromPropertyInspector ? propertyInspectorManager.all[context].action.plugin : pluginManager.plugins[context].uuid;
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
		let svgxmlre = /data:image\/svg\+xml,([^;]+)/;
		let base64re = /data:image\/(apng|avif|gif|jpeg|png|svg\+xml|webp|bmp|x-icon|tiff);base64,([A-Za-z0-9+/]+={0,2})?/;
		if (image) {
			if (svgxmlre.test(image)) image = "data:image/svg+xml;base64," + Buffer.from(svgxmlre.exec(image)[1]).toString("base64");
			if (base64re.test(image)) {
				let e = base64re.exec(image);
				if (!e[2]) image = undefined;
				else image = e[0];
			}
		}
		if (state >= 0) instance.states[state].image = image ?? instance.action.states[state].image;
		else instance.states.forEach((state, index) => state.image = image ?? instance.action.states[index].image);
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

	sendToPropertyInspector({ context, payload }) {
		let instance = getInstanceByContext(context);
		propertyInspectorManager.sendEvent(instance.context, {
			event: "sendToPropertyInspector",
			action: instance.action.uuid,
			context: instance.context,
			payload: payload
		});
	}

	sendToPlugin({ context, payload }) {
		let instance = getInstanceByContext(context);
		pluginManager.sendEvent(instance.action.plugin, {
			event: "sendToPlugin",
			action: instance.action.uuid,
			context: instance.context,
			payload: payload
		});
	}
}

const eventHandler = new EventHandler();
module.exports = { eventHandler };