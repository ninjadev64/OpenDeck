const store = require("./store");

class Action {
	constructor(name, uuid, plugin, tooltip, icon, propertyInspector, controllers) {
		this.name = name;
		this.uuid = uuid;
		this.plugin = plugin;
		this.tooltip = tooltip;
		this.icon = icon;
		this.propertyInspector = propertyInspector;
		this.controllers = controllers;
	}
}

var keys = store.get("keys");
var sliders = store.get("sliders");

var allActions = { };
var categories = { };

function updateKey(key, action) {
	const { eventHandler } = require("./event");
	const { propertyInspectorManager } = require("./propertyinspector");
	if (action == undefined) {
		eventHandler.willDisappear(key);
		propertyInspectorManager.unregister(key);
		keys[key] = undefined;
	} else {
		keys[key] = allActions[action];
		eventHandler.willAppear(key);
		propertyInspectorManager.register(key);
	}
	store.set("keys", keys);
}
function updateSlider(slider, action) {
	const { serialInterface } = require("./serial");
	serialInterface.lastSliders[slider] = 0;
	sliders[slider] = allActions[action];
	store.set("sliders", sliders);
}

module.exports = { keys, sliders, allActions, categories, Action, updateKey, updateSlider };