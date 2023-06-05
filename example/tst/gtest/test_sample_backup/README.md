## 1. Prepare SUT as below
### sample.c
```
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
    static uint8_t pinLeft = 0U;
    static uint8_t pinRight = 0U;
    LOCAL_STATIC_VARIABLE(pinLeft);
    LOCAL_STATIC_VARIABLE(pinRight);
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
```

## 2. Generate gtest files using cyagen
- open SUT file (.c)
- press **F1** and select **cyagen: generate files...**
- select **gtest**

## 3. Check the generated files
- press **F1** and select **Preferences: Open User Settings(JSON)**
- find **vscode-cyagen.templates** and **outputFolder** for **gtest** as below
```
"outputFolder": "${fileDirname}/../tst/gtest/test_@sourcename@"
```
- **${fileDirname}** is the directory where SUT is located
- **@sourcename@** is the filename (witout extension) of SUT

## 5. Complete UT script
- open **test_@sourcename@.cc** (test_sample.cc in this example)
- add missing stub functions within **MANUAL SECTION** as below
```
  /// stub functions
  // MANUAL SECTION: d93bc5d9-008c-5b86-9858-dba6854f4266
  MOCK_METHOD(uint32_t, getCurrentTime, ());
  MOCK_METHOD(void, controlPin, (uint8_t, uint8_t));
  // MANUAL SECTION END
```
```
extern "C"
{
  /// stub functions; use Stub::getInstance().stub_func()
  // MANUAL SECTION: 7c512ffc-1a83-57e3-8c38-ddde70ff83be
  uint32_t getCurrentTime()
  {
    return Stub::getInstance().getCurrentTime();
  }
  void controlPin(uint8_t pin, uint8_t high)
  {
    return Stub::getInstance().controlPin(pin, high);
  }
  // MANUAL SECTION END
}
```
- complete test functions within **MANUAL SECTION** as below
```
/// define a test case for the move() function
TEST(sample, move) {
  // MANUAL SECTION: afd5313d-1dfd-5f93-8fba-47d757174a4d
  // prepare precondition & expected calls
  ::testing::Sequence seq;
  EXPECT_CALL(Stub::getInstance(), getCurrentTime()).Times(1).InSequence(seq).WillOnce(::testing::Return(10));
  EXPECT_CALL(Stub::getInstance(), controlMotor()).Times(1).InSequence(seq).WillRepeatedly([]() { pinUpdated = 1U; });

  // call SUT
  move(Forward, 10);

  // check results
  EXPECT_EQ(currDir, Forward);
  EXPECT_EQ(timeLeft, 10);
  EXPECT_EQ(lastTimestamp, 10);
  EXPECT_EQ(pinUpdated, 1U);
  // MANUAL SECTION END
}
/// define a test case for the checkTimeout() function
TEST(sample, checkTimeout) {
  // MANUAL SECTION: 7968e4aa-0cb9-5a90-8491-21484676bf99
  // prepare precondition & expected calls
  ::testing::Sequence seq;
  lastTimestamp = 0U;
  timeLeft = 10;
  currDir = Forward;
  EXPECT_CALL(Stub::getInstance(), getCurrentTime()).Times(1).InSequence(seq).WillOnce(::testing::Return(10));
  EXPECT_CALL(Stub::getInstance(), controlMotor()).Times(1).InSequence(seq).WillRepeatedly([]() { pinUpdated = 1U; });

  // call SUT
  checkTimeout();

  // check results
  EXPECT_EQ(currDir, Idle);
  // MANUAL SECTION END
}
```
- adapt CMakeLists.txt by adding missing include path

## 6. Build test
```
$ cmake -S . -B build
$ cmake --build build
```

## 7. Run test
```
$ cd build && ctest --rerun-failed --output-on-failure
```

Refer to the [link](https://google.github.io/googletest/quickstart-cmake.html)
