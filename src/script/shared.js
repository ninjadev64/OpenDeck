const store = require("./store");

class Action {
	constructor(name, uuid, plugin, tooltip, icon, propertyInspector) {
		this.name = name;
		this.uuid = uuid;
		this.plugin = plugin;
		this.tooltip = tooltip;
		this.icon = icon;
		this.propertyInspector = propertyInspector;
	}
}

var keys = store.get("keys");

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

module.exports = { keys, allActions, categories, Action, updateKey };