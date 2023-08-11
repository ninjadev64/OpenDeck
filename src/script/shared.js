const store = require("./store");

class Action {
	constructor(name, uuid, plugin, tooltip, icon, propertyInspector, controllers, states) {
		this.name = name;
		this.uuid = uuid;
		this.plugin = plugin;
		this.tooltip = tooltip;
		this.icon = icon;
		this.propertyInspector = propertyInspector;
		this.controllers = controllers;
		this.states = states;
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
	constructor(action, context, type) {
		this.action = action;
		this.context = context;
		this.type = type;
		this.index = type == "Keypad" ? context : type == "Encoder" ? parseInt(context.slice(1)) : undefined;
		this.state = 0;
		this.states = JSON.parse(JSON.stringify(action.states));
	}
}

var keys = store.get("keys");
var sliders = store.get("sliders");

var allActions = {};
var categories = {};

function updateKey(key, instance) {
	const { eventHandler } = require("./event");
	const { propertyInspectorManager } = require("./propertyinspector");
	store.set("actionSettings." + key, {});
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
	const { eventHandler } = require("./event");
	const { serialInterface } = require("./serial");
	const { propertyInspectorManager } = require("./propertyinspector");
	store.set("actionSettings." + slider, {});
	let index = parseInt(slider.slice(1));
	serialInterface.lastSliders[index] = 0;
	if (instance == undefined) {
		eventHandler.willDisappear(sliders[index]);
		propertyInspectorManager.unregister(sliders[index]);
		sliders[index] = undefined;
	} else {
		sliders[index] = instance;
		eventHandler.willAppear(instance);
		propertyInspectorManager.register(instance);
	}
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