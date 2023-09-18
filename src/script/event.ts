import { Device } from "./devices";
import { getMainWindow } from "./main";
import { pluginManager } from "./plugins";
import { propertyInspectorManager } from "./propertyinspector";
import { ActionInstance, getCoordinatesByContext, getInstanceByContext, getProfile, parseContext, updateProfile } from "./shared";
import store from "./store";

import { shell } from "electron";
import log from "electron-log";

class EventHandler {
	updateState(instance: ActionInstance): void {
		let context = parseContext(instance.context);
		getProfile(context.device)[context.type][context.position][context.index] = instance;
		updateProfile(context.device);

		require("./devices").deviceManager.setImage(instance.device, instance.position, instance.states[instance.state].image);

		let window = getMainWindow();
		if (!window || window.isDestroyed()) return;
		getMainWindow().webContents.send("updateState", instance.context, instance);
	}

	// Outbound events
	// Reference: https://docs.elgato.com/sdk/plugins/events-received

	keyDown(device: string, key: number): void {
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

	keyUp(device: string, key: number): void {
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

	dialRotate(device: string, slider: number, value: number): void {
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

	willAppear(instance: ActionInstance): void {
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
		require("./devices").deviceManager.setImage(instance.device, instance.position, instance.states[instance.state].image);
	}

	willDisappear(instance: ActionInstance): void {
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

	deviceDidConnect(id: string, device: Device): void {
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

	deviceDidDisconnect(id: string): void {
		pluginManager.sendGlobalEvent({
			event: "deviceDidDisconnect",
			device: id
		});
	}

	applicationDidLaunch(application: string, plugin: string): void {
		pluginManager.sendEvent(plugin, {
			event: "applicationDidLaunch",
			payload: {
				application
			}
		});
	}

	applicationDidTerminate(application: string, plugin: string): void {
		pluginManager.sendEvent(plugin, {
			event: "applicationDidTerminate",
			payload: {
				application
			}
		});
	}

	propertyInspectorDidAppear(instance: ActionInstance): void {
		pluginManager.sendEvent(instance.action.plugin, {
			event: "propertyInspectorDidAppear",
			action: instance.action.uuid,
			context: instance.context,
			device: instance.device
		});
	}

	propertyInspectorDidDisappear(instance: ActionInstance): void {
		pluginManager.sendEvent(instance.action.plugin, {
			event: "propertyInspectorDidDisappear",
			action: instance.action.uuid,
			context: instance.context,
			device: instance.device
		});
	}

	didReceiveSettings(instance: ActionInstance, propertyInspector: boolean): void {
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

	didReceiveGlobalSettings(plugin: string, propertyInspector: boolean): void {
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

	setSettings({ context, payload }: { context: string, payload: object }, fromPropertyInspector: boolean): void {
		let instance = getInstanceByContext(context);
		instance.settings = payload;
		this.updateState(instance);
		this.didReceiveSettings(instance, !fromPropertyInspector);
	}

	getSettings({ context }: { context: string }, fromPropertyInspector: boolean): void {
		this.didReceiveSettings(getInstanceByContext(context), fromPropertyInspector);
	}

	setGlobalSettings({ context, payload }: { context: string, payload: object }, fromPropertyInspector: boolean): void {
		let plugin = fromPropertyInspector ? propertyInspectorManager.all[context].action.plugin : pluginManager.plugins[context].uuid;
		store.set("pluginSettings." + plugin.replaceAll(".", "¬"), payload);
		this.didReceiveGlobalSettings(plugin, !fromPropertyInspector);
	}

	getGlobalSettings({ context }: { context: string }, fromPropertyInspector: boolean): void {
		let plugin = fromPropertyInspector ? propertyInspectorManager.all[context].action.plugin : pluginManager.plugins[context].uuid;
		this.didReceiveGlobalSettings(plugin, fromPropertyInspector)
	}

	openUrl({ payload: { url } }: { payload: { url: string } }): void {
		shell.openExternal(url);
	}

	logMessage({ payload: { message } }: { payload: { message: string } }): void {
		log.debug(message);
	}

	setTitle({ context, payload: { title, state } }: { context: string, payload: { title: string, state: number } }): void {
		let instance = getInstanceByContext(context);
		if (state) instance.states[state].title = title;
		else instance.states.forEach((state) => state.title = title);
		this.updateState(instance);
	}

	setImage({ context, payload: { image, state } }: { context: string, payload: { image: string, state: number } }): void {
		let instance = getInstanceByContext(context);
		let svgxmlre = /^data:image\/svg\+xml,(.+)/;
		let base64re = /^data:image\/(apng|avif|gif|jpeg|png|svg\+xml|webp|bmp|x-icon|tiff);base64,([A-Za-z0-9+/]+={0,2})?/;
		if (image) {
			if (svgxmlre.test(image)) image = "data:image/svg+xml;base64," + Buffer.from(decodeURIComponent(svgxmlre.exec(image)[1].replace(/\;$/, ""))).toString("base64");
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

	setState({ context, payload: { state } }: { context: string, payload: { state: number }}): void {
		let instance = getInstanceByContext(context);
		instance.state = state;
		this.updateState(instance);
	}

	showAlert({ context }: { context: string }): void {
		getMainWindow().webContents.send("showAlert", context);
	}

	showOk({ context }: { context: string }): void {
		getMainWindow().webContents.send("showOk", context);
	}

	sendToPropertyInspector({ context, payload }: { context: string, payload: object }): void {
		let instance = getInstanceByContext(context);
		propertyInspectorManager.sendEvent(instance.context, {
			event: "sendToPropertyInspector",
			action: instance.action.uuid,
			context: instance.context,
			payload: payload
		});
	}

	sendToPlugin({ context, payload }: { context: string, payload: object }): void {
		let instance = getInstanceByContext(context);
		pluginManager.sendEvent(instance.action.plugin, {
			event: "sendToPlugin",
			action: instance.action.uuid,
			context: instance.context,
			payload: payload
		});
	}
}

export const eventHandler = new EventHandler();