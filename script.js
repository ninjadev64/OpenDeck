const { SerialPort } = require('serialport');
const { ReadlineParser } = require('@serialport/parser-readline');
const port = new SerialPort({ path: '/dev/ttyACM0', baudRate: 57600 });
const parser = port.pipe(new ReadlineParser({ delimiter: '\r\n' }));
const display = document.getElementById("button");

var callbacks = {
	1: red,
	2: green,
	3: blue
}

function text(dat) {
	display.innerText = `Button ${JSON.parse(dat).button} was pressed!`;
}

function red(_) { document.body.style.backgroundColor = "red"; }
function green(_) { document.body.style.backgroundColor = "green"; }
function blue(_) { document.body.style.backgroundColor = "blue"; }

parser.on("data", function (dat) {
	/*console.log(`${dat}: ${*/callbacks[JSON.parse(dat).button](dat);
});