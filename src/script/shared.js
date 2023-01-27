const store = require("./store");

class Action {
	constructor(name, uuid, plugin, tooltip, icon) {
		this.name = name;
		this.uuid = uuid;
		this.plugin = plugin;
		this.tooltip = tooltip;
		this.icon = icon;
	}
}

var keys = store.get("keys");

var allActions = { };
var categories = { };

function updateKey(key, action) {
	const { eventHandler } = require("./event");
	eventHandler.willDisappear(key);
	keys[key] = allActions[action];
	store.set("keys", keys);
	eventHandler.willAppear(key);
}

module.exports = { keys, allActions, categories, Action, updateKey };