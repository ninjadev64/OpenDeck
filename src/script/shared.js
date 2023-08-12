const store = require("./store");

const log = require("electron-log");
const dialog = require("dialog");
const { exit } = require("process");

function parseContext(context) {
	context = context.split(".");
	return { profile: context[0], type: context[1], position: parseInt(context[2]), index: parseInt(context[3]) };
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
	constructor(action, profile, type, position, index) {
		this.action = action;

		this.profile = profile;
		this.type = type;
		this.position = position;
		this.index = index;
		this.context = `${profile}.${type}.${position}.${index}`;
		
		this.state = 0;
		this.states = JSON.parse(JSON.stringify(action.states));

		this.settings = {};
	}
}

var allActions = {};
var categories = {};

var currentProfile = store.get("profiles." + store.get("selectedProfile"));

function updateSlot(context, instance) {
	context = parseContext(context);
	const { eventHandler } = require("./event");
	const { propertyInspectorManager } = require("./propertyinspector");
	if (context.type == "slider") {
		const { serialInterface } = require("./serial");
		serialInterface.lastSliders[index] = 0;
	}
	let position = currentProfile[context.type][context.position];
	if (instance == undefined) {
		eventHandler.willDisappear(position[context.index]);
		propertyInspectorManager.unregister(position[context.index]);
		position[context.index] = null;
	} else {
		position[context.index] = instance;
		eventHandler.willAppear(instance);
		propertyInspectorManager.register(instance);
	}
	store.set("profiles." + store.get("selectedProfile"), currentProfile);
}

function getInstanceByContext(context) {
	context = parseContext(context);
	return currentProfile[context.type][context.position][context.index];
}

function getCoordinatesByContext(context) {
	context = parseContext(context);
	return {
		row: Math.floor(context.position / 3),
		column: context.position % 3
	}
}

function error(message, fatal) {
	log.error(message);
	dialog.err(message, "Error - OceanDesktop", () => {
		if (fatal) exit(1);
	});
}

module.exports = { allActions, categories, currentProfile, Action, ActionInstance, ActionState, updateSlot, parseContext, getInstanceByContext, getCoordinatesByContext, error };