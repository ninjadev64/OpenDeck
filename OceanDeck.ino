#include "ArduinoJson.h";

int lastKey;
int lastSlider0;
int lastSlider1;

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

  int key = 0;
  /**/ if (digitalRead(2) == LOW) key = 1;
  else if (digitalRead(3) == LOW) key = 2;
  else if (digitalRead(4) == LOW) key = 3;
  if (key != lastKey) {
    lastKey = key;
    doc["key"] = key;
  }

  int s0 = round(analogRead(A0) / 6.81);
  if (s0 != lastSlider0) {
    lastSlider0 = s0;
    doc["slider0"] = s0;
  }

  /*
  int s1 = round(analogRead(A1) / 6.81);
  if (s1 != lastSlider1) {
    lastSlider1 = s1;
    doc["slider1"] = s1;
  }
  */

  String output = "";
  serializeJson(doc, output);
  if (output == "null") return;
  Serial.println(output);

  delay(50);
}
