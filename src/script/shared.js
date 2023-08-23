const store = require("./store");

const log = require("electron-log");
const dialog = require("dialog");
const { exit } = require("process");
const { existsSync } = require("fs");

function createUniqueId() {
	return Date.now().toString(36) + Math.random().toString(36).substring(2);
}

function parseContext(context) {
	context = context.split(".");
	return { device: context[0], profile: context[1], type: context[2], position: parseInt(context[3]), index: parseInt(context[4]) };
}

function getIcon(path) {
	if (existsSync(path + ".svg")) return path + ".svg";
	else if (existsSync(path + "@2x.png")) return path + "@2x.png";
	else return path + ".png";
}

class Action {
	constructor(name, uuid, plugin, tooltip, icon, propertyInspector, controllers, states, visibleInActionsList) {
		this.name = name;
		this.uuid = uuid;
		this.plugin = plugin;
		this.tooltip = tooltip;
		this.icon = icon;
		this.propertyInspector = propertyInspector;
		this.controllers = controllers;
		this.states = states;
		this.visibleInActionsList = visibleInActionsList;
	}
}

class ActionState {
	constructor(data, actionDefaultName) {
		this.image = data.Image;
		this.multiActionImage = data.MultiActionImage ?? this.image;
		this.name = data.Name ?? actionDefaultName;
		this.title = data.Title ?? actionDefaultName;
		this.showTitle = data.ShowTitle ?? true;
		this.titleColour = data.TitleColor ?? "#f2f2f2";
		this.titleAlignment = data.TitleAlignment ?? "middle";
		this.titleFontStyle = data.FontStyle ?? "Regular";
		this.titleFontSize = data.FontSize ?? 16;
		this.titleFontUnderline = data.FontUnderline ?? false;
	}
}

class ActionInstance {
	constructor(action, device, profile, type, position, index) {
		this.action = action;

		this.device = device;
		this.profile = profile;
		this.type = type;
		this.position = position;
		this.index = index;
		this.context = `${device}.${profile}.${type}.${position}.${index}`;
		
		this.state = 0;
		this.states = JSON.parse(JSON.stringify(action.states));

		this.settings = {};
	}
}

var allActions = {};
var categories = {};

var currentProfiles = {};
for (const [id, settings] of Object.entries(store.get("devices"))) {
	currentProfiles[id] = settings.profiles[settings.selectedProfile];
}

function setProfile(device, id) {
	const { eventHandler } = require("./event");
	currentProfiles[device].key.forEach((slot) => slot.forEach((instance) => {
		if (instance) eventHandler.willDisappear(instance);
	}));
	let devices = store.get("devices");
	devices[device].selectedProfile = id;
	currentProfiles[device] = devices[device].profiles[id];
	store.set("devices", devices);
}

function getProfile(device) {
	return currentProfiles[device];
}

function updateProfile(device) {
	let devices = store.get("devices");
	devices[device].profiles[devices[device].selectedProfile] = getProfile(device);
	store.set("devices", devices);
}

function updateSlot(context, instance) {
	context = parseContext(context);
	const { eventHandler } = require("./event");
	const { propertyInspectorManager } = require("./propertyinspector");
	if (context.type == "slider") {
		const { deviceManager } = require("./devices");
		deviceManager.devices[context.device].lastSliders[context.index] = 0;
	}
	let position = getProfile(context.device)[context.type][context.position];
	if (instance == undefined) {
		eventHandler.willDisappear(position[context.index]);
		propertyInspectorManager.unregister(position[context.index]);
		position[context.index] = null;
	} else {
		position[context.index] = instance;
		eventHandler.willAppear(instance);
		propertyInspectorManager.register(instance);
	}
	updateProfile(context.device);
}

function getInstanceByContext(context) {
	context = parseContext(context);
	return getProfile(context.device)[context.type][context.position][context.index];
}

function getCoordinatesByContext(context) {
	context = parseContext(context);
	const { deviceManager } = require("./devices");
	let device = deviceManager.devices[context.device];
	return {
		row: Math.floor(context.position / device.rows),
		column: context.position % device.columns
	}
}

function error(message, fatal) {
	log.error(message);
	dialog.err(message, "Error - OpenDeck", () => {
		if (fatal) exit(1);
	});
}

module.exports = { allActions, categories, Action, ActionInstance, ActionState, createUniqueId, parseContext, getIcon, setProfile, getProfile, updateProfile, updateSlot, getInstanceByContext, getCoordinatesByContext, error };