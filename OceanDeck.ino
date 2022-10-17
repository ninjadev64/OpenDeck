#include "ArduinoJson.h";

int lastPressed;

void setup() {
  Serial.begin(57600);
}

void loop() {
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
