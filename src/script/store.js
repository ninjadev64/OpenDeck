let randomDefaultProfileId = Date.now().toString(36) + Math.random().toString(36).substring(2);
const schema = {
	profiles: {
		type: "object",
		default: {
			[randomDefaultProfileId]: {
				name: "Default",
				key: [ [ null ], [ null ], [ null ], [ null ], [ null ], [ null ], [ null ], [ null ], [ null ] ],
				slider: [ [ null ], [ null ] ]
			}
		}
	},
	selectedProfile: {
		type: "string",
		default: randomDefaultProfileId
	},
	serialPort: {
		type: "string"
	},
	webSocketPort: {
		type: "number",
        minimum: 0,
        maximum: 65535,
		default: 57116
	},
	propertyInspectorPort: {
		type: "number",
        minimum: 0,
        maximum: 65535,
		default: 57117
	},
	userDataPath: {
		type: "string"
	},
	autoLaunch: {
		type: "boolean",
		default: true
	},
	pluginSettings: {
		type: "object",
		default: {}
	},
	bundleIDs: {
		type: "object",
		default: {}
	}
}

const Store = require("electron-store");
const store = new Store({ schema });

module.exports = store;