#include "ArduinoJson.h";

int lastPressed;

void setup() {
  pinMode(2, INPUT);
  pinMode(3, INPUT);
  pinMode(4, INPUT);

  digitalWrite(2, HIGH);
  digitalWrite(3, HIGH);
  digitalWrite(4, HIGH);

  Serial.begin(57600);
}

void loop() {
  StaticJsonDocument<16> doc;
  /**/ if (digitalRead(2) == LOW) doc["button"] = 1;
  else if (digitalRead(3) == LOW) doc["button"] = 2;
  else if (digitalRead(4) == LOW) doc["button"] = 3;
  else doc["button"] = 0;
  
  if (lastPressed == doc["button"]) return;
  lastPressed = doc["button"];
  
  String output = "";
  serializeJson(doc, output);
  Serial.println(output);

  delay(50);
}
