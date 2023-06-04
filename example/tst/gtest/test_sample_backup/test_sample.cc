/// this is generated googletest script based on sample.c

#include <gtest/gtest.h>
#include <gmock/gmock.h>
#include <string>
#include <map>

//////////////////////////////////////////////////////////////////////////////
/// to access a local static variable from unit test script
/// 1. add LOCAL_STATIC_VARIABLE(variable) after the declaration of the variable in the function
/// 2. use ACCESS_LOCAL_STATIC_VARIABLE(fn, vn, vt) in unit test script
std::map<std::pair<std::string, std::string>, void*> _local_static_variables;
#define LOCAL_STATIC_VARIABLE(var) _local_static_variables[std::make_pair(__func__, #var)] = &var
#define ACCESS_LOCAL_STATIC_VARIABLE(fn, vn, vt) *static_cast<vt *>([](){ try { return _local_static_variables.at(std::make_pair(#fn,#vn)); } catch (const std::exception& e) { fprintf(stderr,"(%s,%s) not existed\n",#fn,#vn); return (void*)nullptr; } }())
//////////////////////////////////////////////////////////////////////////////

extern "C"
{
  #include <stdio.h>
  #include <stdint.h>
  // MANUAL SECTION: 5bb47141-e026-5086-bd7f-cd52a657b4c6
  // MANUAL SECTION END
}

/// Stub class for stub functions
class Stub
{
public:
  Stub(Stub const &) = delete;
  Stub(Stub &&) = delete;
  Stub &operator=(Stub const &) = delete;
  Stub &operator=(Stub &&) = delete;
  static auto &getInstance()
  {
    static Stub st;
    return st;
  }
  // move calls controlMotor
  MOCK_METHOD(void, controlMotor, ());
  // checkTimeout calls controlMotor
  MOCK_METHOD(void, controlMotor, ());
  

  /// stub functions
  // MANUAL SECTION: d93bc5d9-008c-5b86-9858-dba6854f4266
  MOCK_METHOD(uint32_t, getCurrentTime, ());
  MOCK_METHOD(void, controlPin, (uint8_t, uint8_t));
  MOCK_METHOD(void, controlMotor, ());
  // MANUAL SECTION END
private:
  Stub(){}
};

extern "C"
{
  // MANUAL SECTION: 4c4e914f-8bc0-58f9-9c3b-17ee4e33cfd0
// MANUAL SECTION END

  // include SUT
  #include "sample.c"

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

/// usage of EXPECT_CALL() for stub functions
// EXPECT_CALL(Stub::getInstance(), stub_func1()).WillOnce(::testing::Return(1));

// MANUAL SECTION: 35d1701a-c77d-57d9-bed5-d756ffe01cc0
// MANUAL SECTION END

/// define a test case for the controlMotor function
TEST(sample, controlMotor) {
  // MANUAL SECTION: cf139424-4850-5640-aab9-dc3ae9584c7d
  EXPECT_CALL(Stub::getInstance(), controlPin(MOTOR_LEFT_PIN, 0));
  EXPECT_CALL(Stub::getInstance(), controlPin(MOTOR_RIGHT_PIN, 0));
  currDir = Idle;
  controlMotor();
  ///
  EXPECT_CALL(Stub::getInstance(), controlPin(MOTOR_LEFT_PIN, 1));
  EXPECT_CALL(Stub::getInstance(), controlPin(MOTOR_RIGHT_PIN, 1));
  currDir = Forward;
  controlMotor();
  // MANUAL SECTION END
}
/// define a test case for the move function
TEST(sample, move) {
  // MANUAL SECTION: afd5313d-1dfd-5f93-8fba-47d757174a4d
  ::testing::Sequence seq;
  EXPECT_CALL(Stub::getInstance(), getCurrentTime()).Times(1).InSequence(seq).WillOnce(::testing::Return(10));
  EXPECT_CALL(Stub::getInstance(), controlMotor()).Times(1).InSequence(seq).WillRepeatedly([]()
                                                                                           { pinUpdated = 1U; });
  move(Forward, 10);
  EXPECT_EQ(currDir, Forward);
  EXPECT_EQ(timeLeft, 10);
  EXPECT_EQ(lastTimestamp, 10);
  EXPECT_EQ(pinUpdated, 1U);
  // MANUAL SECTION END
}
/// define a test case for the checkTimeout function
TEST(sample, checkTimeout) {
  // MANUAL SECTION: 7968e4aa-0cb9-5a90-8491-21484676bf99
  ::testing::Sequence seq;
  lastTimestamp = 0U;
  timeLeft = 10;
  currDir = Forward;
  EXPECT_CALL(Stub::getInstance(), getCurrentTime()).Times(1).InSequence(seq).WillOnce(::testing::Return(9));
  EXPECT_CALL(Stub::getInstance(), controlMotor()).Times(1).InSequence(seq).WillRepeatedly([]()
                                                                                           { pinUpdated = 1U; });
  checkTimeout();
  EXPECT_EQ(currDir, Idle);
  // MANUAL SECTION END
}


int main(int argc, char **argv) {
  ::testing::InitGoogleTest(&argc, argv);
  return RUN_ALL_TESTS();
}