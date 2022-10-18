#include "ArduinoJson.h";

int lastPressed;
char incomingMessage[64];
int incomingMessagePos = 0;

void setup() {
  Serial.begin(57600);
}

void loop() {
  if (Serial.available() > 0) {
    char inByte = Serial.read();
    if (inByte != '\n') {
      incomingMessage[incomingMessagePos] = inByte;
      incomingMessagePos++;
    } else {
      StaticJsonDocument<64> doc;
      deserializeJson(doc, incomingMessage);
      analogWrite(9, doc["red"].as<uint8_t>());
      analogWrite(10, doc["green"].as<uint8_t>());
      analogWrite(11, doc["blue"].as<uint8_t>());
      incomingMessagePos = 0;
      memset(incomingMessage, 0, sizeof(incomingMessage));
    }
  }
  
  StaticJsonDocument<16> doc;
  /**/ if (digitalRead(2) == HIGH) doc["button"] = 1;
  else if (digitalRead(3) == HIGH) doc["button"] = 2;
  else if (digitalRead(4) == HIGH) doc["button"] = 3;
  else {
    lastPressed = 0;
    return;
  }
  
  if (lastPressed == doc["button"]) return;
  lastPressed = doc["button"];
  
  String output = "";
  serializeJson(doc, output);
  Serial.println(output);

  delay(50);
}
