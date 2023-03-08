const schema = {
	keys: {
		type: "array",
        default: [
            undefined, undefined, undefined, undefined, undefined, undefined, undefined, undefined, undefined
        ]
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
	userDataPath: {
		type: "string"
	}
}

const Store = require("electron-store");
const store = new Store({ schema });

module.exports = store;