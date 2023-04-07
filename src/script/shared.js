const store = require("./store");
const { ipcMain } = require("electron");

class Action {
	constructor(name, uuid, plugin, tooltip, icon, propertyInspector, controllers) {
		this.name = name;
		this.uuid = uuid;
		this.plugin = plugin;
		this.tooltip = tooltip;
		this.icon = icon;
		this.propertyInspector = propertyInspector;
		this.controllers = controllers;
		this.states = [];
	}
}

class ActionState {
	constructor(action, data) {
		this.image = data.Image == "actionDefaultImage" ? action.icon : data.Image;
		this.name = data.Name;
		this.title = data.Title;
	}
}

class ActionInstance {
	constructor(action, context, type) {
		this.action = action;
		this.context = context;
		this.type = type;
		this.index = type == "Keypad" ? context : type == "Encoder" ? parseInt(context.slice(1)) : undefined;
		this.state = 0;
	}
}

var keys = store.get("keys");
var sliders = store.get("sliders");

var allActions = { };
var categories = { };

function updateKey(key, instance) {
	const { eventHandler } = require("./event");
	const { propertyInspectorManager } = require("./propertyinspector");
	if (instance == undefined) {
		eventHandler.willDisappear(keys[key]);
		propertyInspectorManager.unregister(keys[key]);
		keys[key] = undefined;
	} else {
		keys[key] = instance;
		eventHandler.willAppear(instance);
		propertyInspectorManager.register(instance);
	}
	store.set("keys", keys);
}
function updateSlider(slider, instance) {
	const { serialInterface } = require("./serial");
	const { propertyInspectorManager } = require("./propertyinspector");
	let index = parseInt(slider.slice(1));
	serialInterface.lastSliders[index] = 0;
	if (instance == undefined) {
		propertyInspectorManager.unregister(sliders[index]);
	} else {
		propertyInspectorManager.register(instance);
	}
	sliders[index] = instance;
	store.set("sliders", sliders);
}

function getInstanceByContext(context) {
	if (context.toString().startsWith("s")) {
		return sliders[parseInt(context.slice(1))];
	} else {
		return keys[parseInt(context)];
	}
}

module.exports = { keys, sliders, allActions, categories, Action, ActionInstance, ActionState, updateKey, updateSlider, getInstanceByContext };