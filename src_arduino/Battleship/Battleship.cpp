#include "Arduino.h"
#include "Battleship.h"
#include <FastLED_NeoPixel.h>

#define LEFT_THRESHOLD  400
#define RIGHT_THRESHOLD 800
#define UP_THRESHOLD    400
#define DOWN_THRESHOLD  800

int xValue = 0 ; // To store value of the X axis
int yValue = 0 ; // To store value of the Y axis

bool isMyTurn = false;
bool isEnd = false;
bool isReset = false;
bool changeTurnReceived = false; // True if other arduino has received the change turn message
bool endReceived = false;
bool resetReceived = false;
bool firstTurn = false;

String myShipsMsg = ""; // Contains this players ships as a msg to send to the other Arduino
String theirShipsMsg = ""; // Contains the opponents ships as a msg to send to the PC

// Input messages from pc
char pcInMsgReset = '0'; // Pc tells arduino to reset, msg is forwarded to the other pc
char pcInMsgShips = '1'; // Pc sends ship positions, message: "1x", where x is 128 bit unsigned intenger
char pcInMsgJoystick = '2'; // Pc requests joystick direction
char pcInMsgHit = '3'; // Pc has hit a cell, message: "3x" where x is a cell from 0-99 on a 10x10 board
char pcInMsgMiss = '4'; // Pc has missed a cell (turn over)
char pcInMsgEnd = '5'; // Pc has hit all ships
char pcInMsgGetShips = '6'; // Pc request the opponents ship positions

// Output messages to pc
char pcOutMsgReset = '0'; // Tell pc to reset game
char pcOutMsgShips = '1'; // Send opponents ships to pc, message: "1x", where x is 128 bit unsigned intenger
char pcOutMsgFire = '2'; // Tell pc to fire 
char pcOutMsgJoystick = '3'; // Send joystick direction to pc, message: "3x,y,z", where x is the x direction and y is the y direction, z is bool for turn
char pcOutMsgEnd = '4'; // Other player has won the game
char pcOutTurn = '5'; // Tell pc that it is its turn

// Communication with other arduino
char msgShips = '1'; // Transfer board as json object, message: "1x", where x is a json object of the board
char msgHit = '2'; // Transfer hit data, message: "2x" where x is a cell from 0-99 on a 10x10 board
char msgTurn = '3'; // Tell other arduino that it is their turn
char msgEnd = '4'; // Tell other arduino that the game has ended
char msgGetShips = '5'; // Request the ship positions from the other arduino
char msgTurnReceived = '6'; // Response when changing turn to confirm that the turn has changed.
char msgReset = '7'; // Transfer reset message
char msgMiss = '8';
char msgEndReceived = '9';
char msgResetReceived = '10';

Battleship::Battleship(bool isServer, int cols, int motorPin, int buttonPin, int vrxPin, int vryPin, bool invertAxis) {
	if (isServer)
	{
		isMyTurn = true;
		firstTurn = true;
	}
	
	_cols = cols;
	_motorPin = motorPin;
	_buttonPin = buttonPin;
	_vrxPin = vrxPin;
	_vryPin = vryPin;
	_invertAxis = invertAxis;

	pinMode(motorPin, OUTPUT);
	pinMode(buttonPin, INPUT_PULLUP);
}

void Battleship::vibrate() {
  digitalWrite(_motorPin, HIGH);
  delay(500);
  digitalWrite(_motorPin, LOW);
}

int Battleship::arrayToStripIndex(int i) {
  // Convert array index to S index;
  if (((int)floor(i/_cols))%2 == 0) {
    return i;
  } else {
    return floor(i/_cols)*_cols + (_cols-1)-i%_cols;
  }
}

void Battleship::sendJoystickDirection() {
	// read analog X and Y analog values
	xValue = analogRead(_vrxPin);
	yValue = analogRead(_vryPin);

	int xToSend = 0;
	int yToSend = 0;
	
	// check left/right
	if (xValue < LEFT_THRESHOLD || xValue > RIGHT_THRESHOLD)
		xToSend = uInt10ToInt8(xValue);

	// check up/down
	if (yValue < UP_THRESHOLD || yValue > DOWN_THRESHOLD)
		yToSend = uInt10ToInt8(yValue);

	// invert axis if the joystick was rotated 180deg
	if (_invertAxis) {
		xToSend *= -1;
		yToSend *= -1;
	}

	// send x and y to pc as: 
	String msg = (String)pcOutMsgJoystick + xToSend + "," + yToSend + "," + (isMyTurn ? '1' : '0');
	Serial.println(msg);
}

void Battleship::run(FastLED_NeoPixel_Variant strip) {
	// button output
	if (digitalRead(_buttonPin) == 0) {
		Serial.println(pcOutMsgFire);
	}

	// handle input from pc
	if (Serial.available()) {
		String msg = Serial.readStringUntil('\n');
		if (msg[0] == pcInMsgReset) {
			resetGame(strip);
			Serial1.println(msgReset);
			isReset = true;
		} else if (msg[0] == pcInMsgShips) {
			myShipsMsg = msgShips + msg.substring(1);
			// tell pc that it has the starting turn
			if (isMyTurn)
			{
				Serial.println(pcOutTurn);
			}
		} else if (msg[0] == pcInMsgJoystick) {
			sendJoystickDirection();
		} else if (msg[0] == pcInMsgHit) {
			Serial1.println(msgHit + msg.substring(1));
			transferTurn();
			vibrate();
		} else if (msg[0] == pcInMsgMiss) {
			Serial1.println(msgMiss + msg.substring(1));
			transferTurn();
		} else if (msg[0] == pcInMsgEnd) {
			Serial1.println(msgEnd);
			isEnd = true;
		} else if (msg[0] == pcInMsgGetShips) {
			// send opponents ships to PC, otherwise request opponents ships from other arduino
			if (theirShipsMsg.length() > 1)
			{
				Serial.println(theirShipsMsg);
			} else {
				Serial1.println(msgGetShips);
			}
		}
	}
	// handle input from other arduino
	if (Serial1.available()) {
		String msg = Serial1.readStringUntil('\n');
		if (msg[0] == msgReset) {
			resetGame(strip);
			Serial.println(pcOutMsgReset);
			Serial1.println(msgResetReceived);
		} else if (msg[0] == msgShips) {
			// save opponents ships
			theirShipsMsg = pcOutMsgShips + msg.substring(1);
		} else if (msg[0] == msgHit) {
			int index = msg.substring(1).toInt();
			strip.setPixelColor(arrayToStripIndex(index), 255, 0, 0);
			strip.show();
			isMyTurn = true;
			vibrate();
		} else if (msg[0] == msgMiss) {
			int index = msg.substring(1).toInt();
			strip.setPixelColor(arrayToStripIndex(index), 0, 0, 255);
			strip.show();
			isMyTurn = true;
		}
		else if (msg[0] == msgTurn) {
			isMyTurn = true;
			Serial1.println(msgTurnReceived);
		} else if (msg[0] == msgEnd) {
			Serial.println(pcOutMsgEnd);
			Serial1.println(msgEndReceived);
		} else if (msg[0] == msgGetShips) {
			// send ships to opponent, if they have been received from the PC
			if (myShipsMsg.length() > 1)
			{
				Serial1.println(myShipsMsg);
			}
		} else if (msg[0] == msgTurnReceived) {
			changeTurnReceived = true;
		} else if (msg[0] == msgEndReceived) {
			isEnd = false;
		} else if (msg[0] == msgResetReceived) {
			isReset = false;
		}
	}
	// make sure the other arduino changes turn
	if (!changeTurnReceived && !isMyTurn) {
		Serial1.println(msgTurn);
	}
	// make sure the other arduino shows end screen
	if (isEnd) {
		Serial1.println(msgEnd);
	}
	// make sure the other arduino resets
	if (isReset) {
		Serial1.println(msgReset);
	}
}

void Battleship::transferTurn() {
	isMyTurn = false;
	changeTurnReceived = false;
	Serial1.println(msgTurn);
}

void Battleship::resetGame(FastLED_NeoPixel_Variant strip) {
	strip.clear();
	strip.show();
	theirShipsMsg = "";
	myShipsMsg = "";
	isMyTurn = firstTurn;
	isEnd = false;
}

int Battleship::uInt10ToInt8(int n) {
	return floor((n-512)/4)-1;
}