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
	eventHandler.willDisappear(key);
	keys[key] = allActions[action];
	store.set("keys", keys);
	eventHandler.willAppear(key);
	propertyInspectorManager.register(key);
}

module.exports = { keys, allActions, categories, Action, updateKey };