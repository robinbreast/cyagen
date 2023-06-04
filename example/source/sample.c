#include <stdio.h>
#include <stdint.h>

typedef enum
{
    Idle = 0,
    Forward,
    TurnLeft,
    TurnRight,
    MaxDirection
} Direction_t;

#define MOTOR_LEFT_PIN 10
#define MOTOR_RIGHT_PIN 11

static Direction_t currDir = Idle;
static uint32_t timeLeft = 0U;
static uint32_t lastTimestamp = 0U;
static uint8_t pinUpdated = 0U;

extern uint32_t
getCurrentTime();
extern void controlPin(uint8_t pin, uint8_t high);

void controlMotor(void)
{
    if (0U == pinUpdated)
    {
        switch (currDir)
        {
        case Idle:
            controlPin(MOTOR_LEFT_PIN, 0);
            controlPin(MOTOR_RIGHT_PIN, 0);
            break;
        case Forward:
            controlPin(MOTOR_LEFT_PIN, 1);
            controlPin(MOTOR_RIGHT_PIN, 1);
            break;
        case TurnLeft:
            controlPin(MOTOR_LEFT_PIN, 1);
            controlPin(MOTOR_RIGHT_PIN, 0);
            break;
        case TurnRight:
            controlPin(MOTOR_LEFT_PIN, 0);
            controlPin(MOTOR_RIGHT_PIN, 1);
            break;
        default:
            break;
        }
        pinUpdated = 1U;
    }
}

void move(const Direction_t dir, const uint32_t duration)
{
    currDir = dir;
    timeLeft = duration;
    lastTimestamp = getCurrentTime();
    pinUpdated = 0U;
    controlMotor();
}

void checkTimeout(void)
{
    const uint32_t elapsed = getCurrentTime() - lastTimestamp;
    if (timeLeft > elapsed)
        timeLeft -= elapsed;
    else
    {
        timeLeft = 0;
        currDir = Idle;
        pinUpdated = 0U;
        controlMotor();
    }
}
