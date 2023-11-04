// @ts-expect-error
import dialog from "dialog";
import log from "electron-log";
import { existsSync } from "fs";
import { exit } from "process";
import store from "./store";

export function createUniqueId() {
	return Date.now().toString(36) + Math.random().toString(36).substring(2);
}

export type Context = {
	device: string,
	profile: string,
	type: "key" | "slider",
	position: number,
	index: number
};

export function parseContext(context: string): Context {
	let split = context.split(".");
	return { device: split[0], profile: split[1], type: split[2] as ("key" | "slider"), position: parseInt(split[3]), index: parseInt(split[4]) };
}

export function getIcon(path: string): string {
	if (existsSync(path + ".svg")) return path + ".svg";
	else if (existsSync(path + "@2x.png")) return path + "@2x.png";
	else return path + ".png";
}

export class Action {
	name: string;
	uuid: string;
	plugin: string;
	tooltip: string;
	icon: string;
	propertyInspector: string;
	controllers: string[];
	states: ActionState[];
	visibleInActionsList: boolean;

	constructor(
		name: string, uuid: string, plugin: string,
		tooltip: string, icon: string, propertyInspector: string,
		controllers: string[], states: ActionState[], visibleInActionsList: boolean
	) {
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

export class ActionTitle {
	text: string;
	show: boolean;
	colour: string;
	alignment: string;
	style: string;
	size: number;
	underline: boolean;

	constructor(text: string, show: boolean, colour: string, alignment: string, style: string, size: number, underline: boolean) {
		this.text = text;
		this.show = show;
		this.colour = colour;
		this.alignment = alignment;
		this.style = style;
		this.size = size;
		this.underline = underline;
	}
}

export class ActionState {
	image: string;
	multiActionImage: string;
	name: string;
	title: ActionTitle;

	constructor(data: any, actionDefaultName: string) {
		this.image = data.Image;
		this.multiActionImage = data.MultiActionImage ?? this.image;
		this.name = data.Name ?? actionDefaultName;
		this.title = new ActionTitle(
			data.Title ?? "",
			data.ShowTitle ?? true,
			data.TitleColor ?? "#f2f2f2",
			data.TitleAlignment ?? "middle",
			data.FontStyle ?? "Regular",
			data.FontSize ?? 16,
			data.FontUnderline ?? false
		);
	}
}

export class ActionInstance {
	action: Action;
	device: string;
	profile: string;
	type: "key" | "slider";
	position: number;
	index: number;
	context: string;
	state: number;
	states: ActionState[];
	settings: object;

	constructor(action: Action, device: string, profile: string, type: "key" | "slider", position: number, index: number) {
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

export let allActions: { [id: string]: Action } = {};
export let categories: { [name: string]: Action[] } = {};

export type Profile = {
	name: string,
	key: ActionInstance[][],
	slider: ActionInstance[][]
}

export let currentProfiles: { [device: string]: Profile } = {};
for (const [ id, data ] of Object.entries(store.get("devices"))) {
	let settings = data as any;
	currentProfiles[id] = settings.profiles[settings.selectedProfile];
}

export function getProfile(device: string): Profile {
	return currentProfiles[device];
}

export function setProfile(device: string, profile: string): Profile {
	const { eventHandler } = require("./event");
	[].concat(currentProfiles[device].key, currentProfiles[device].slider).forEach((slot) => slot.forEach((instance: ActionInstance) => {
		if (instance) eventHandler.willDisappear(instance);
	}));
	let devices = store.get("devices");
	devices[device].selectedProfile = profile;
	currentProfiles[device] = devices[device].profiles[profile];
	store.set("devices", devices);
	return getProfile(device);
}

export function updateProfile(device: string): Profile {
	let devices = store.get("devices");
	devices[device].profiles[devices[device].selectedProfile] = getProfile(device);
	store.set("devices", devices);
	return getProfile(device);
}

export function updateSlot(context: string, instance: ActionInstance): ActionInstance {
	let details = parseContext(context);
	const { eventHandler } = require("./event");
	const { propertyInspectorManager } = require("./propertyinspector");
	if (details.type == "slider") {
		const { deviceManager } = require("./devices");
		const device = deviceManager.devices[details.device];
		if (device && device.lastSliders) device.lastSliders[details.index] = 0;
	}
	let position = getProfile(details.device)[details.type][details.position];
	if (instance == undefined) {
		eventHandler.willDisappear(position[details.index]);
		propertyInspectorManager.unregister(position[details.index]);
		position[details.index] = null;
	} else {
		position[details.index] = instance;
		eventHandler.willAppear(instance);
		propertyInspectorManager.register(instance);
	}
	updateProfile(details.device);
	return instance;
}

export function getInstanceByContext(context: string): ActionInstance {
	let details = parseContext(context);
	return getProfile(details.device)[details.type][details.position][details.index];
}

export type Coordinates = { row: number; column: number };
export function getCoordinatesByContext(context: string): Coordinates {
	let details = parseContext(context);
	let device = store.get("devices." + details.device);
	return {
		row: Math.floor(details.position / device.rows),
		column: details.position % device.columns
	}
}

export function error(message: any, fatal: boolean): void {
	log.error(message);
	dialog.err(message, "Error - OpenDeck", () => {
		if (fatal) exit(1);
	});
}
