#ifndef Battleship_h
#define Battleship_h

#include "Arduino.h"
#include <FastLED_NeoPixel.h>

class Battleship
{
  public:

    Battleship(bool isServer, int cols, int motorPin, int buttonPin, int vrxPin, int vryPin, bool invertAxis);

	void vibrate();
	int arrayToStripIndex(int i);
	void sendJoystickDirection();
	void transferTurn();
	void run(FastLED_NeoPixel_Variant strip);
	void resetGame(FastLED_NeoPixel_Variant strip);
	char uInt10ToInt8(int n);

  private:
	bool _isServer;
  	int _cols;
	int _motorPin;
	int _buttonPin;
	int _vrxPin;
	int _vryPin;
	bool _invertAxis;
};

#endif