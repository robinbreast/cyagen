#include <stdio.h>
#include <stdint.h>

#if !defined(LOCAL_STATIC_VARIABLE)
# define LOCAL_STATIC_VARIABLE(funcname, datatype, varname, initvalue) static datatype varname = initvalue
#endif //!defined(LOCAL_STATIC_VARIABLE)

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

extern uint32_t getCurrentTime();
extern void controlPin(uint8_t pin, uint8_t high);

void controlMotor(void)
{
    LOCAL_STATIC_VARIABLE(controlMotor, uint8_t, pinLeft, 0U);
    LOCAL_STATIC_VARIABLE(controlMotor, uint8_t, pinRight, 0U);
    if (0U == pinUpdated)
    {
        uint8_t left = 0, right = 0;
        switch (currDir)
        {
        case Idle:
            left = 0U;
            right = 0U;
            break;
        case Forward:
            left = 1U;
            right = 1U;
            break;
        case TurnLeft:
            left = 1U;
            right = 0U;
            break;
        case TurnRight:
            left = 0U;
            right = 1U;
            break;
        default:
            break;
        }
        if (pinLeft != left)
        {
            pinLeft = left;
            controlPin(MOTOR_LEFT_PIN, pinLeft);
            pinUpdated = 1U;
        }
        if (pinRight != right)
        {
            pinRight = right;
            controlPin(MOTOR_RIGHT_PIN, pinRight);
            pinUpdated = 1U;
        }
    }
}

void setDir(const Direction_t dir)
{
    currDir = dir;
}

void move(const Direction_t dir, const uint32_t duration)
{
    setDir(dir);
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
        setDir(Idle);
        pinUpdated = 0U;
        controlMotor();
    }
}
