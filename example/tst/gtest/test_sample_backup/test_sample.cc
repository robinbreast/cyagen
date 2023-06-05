/// this is generated googletest script based on sample.c

#include <gtest/gtest.h>
#include <gmock/gmock.h>
#include <string>
#include <map>

//////////////////////////////////////////////////////////////////////////////
/// to access a local static variable from unit test script
/// LOCAL_STATIC_VARIABLE(datatype, variable) after the declaration of the variable in the function
std::map<std::pair<std::string, std::string>, std::pair<void*, void*>> _local_static_vars;
#define LOCAL_STATIC_VARIABLE(dtype, var) \
  do {\
    _local_static_vars[std::make_pair(__func__, #var)].first = &var;\
    var = *(dtype*)_local_static_vars[std::make_pair(__func__, #var)].second;\
  } while (0)

extern "C"
{
  // MANUAL SECTION: 4c4e914f-8bc0-58f9-9c3b-17ee4e33cfd0
  // MANUAL SECTION END

  // include SUT
  #include "sample.c"
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
  /// nested functions for call sequence checks
  MOCK_METHOD(void, controlMotor, ());
  MOCK_METHOD(void, setDir, (const Direction_t));

  /// stub functions
  // MANUAL SECTION: d93bc5d9-008c-5b86-9858-dba6854f4266
  MOCK_METHOD(uint32_t, getCurrentTime, ());
  MOCK_METHOD(void, controlPin, (uint8_t, uint8_t));
  // MANUAL SECTION END

private:
  Stub(){}
};

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

#define LOCAL_STATIC_VARIABLE_WRITE(fn, vn, val) \
  do {\
    _##fn##_##vn = val;\
  } while (0)
#define LOCAL_STATIC_VARIABLE_READ(fn, vn, vt) *static_cast<vt *>([](){ try { return _local_static_vars.at(std::make_pair(#fn,#vn)).first; } catch (const std::exception& e) { fprintf(stderr,"(%s,%s) not existed\n",#fn,#vn); return (void*)nullptr; } }())
//////////////////////////////////////////////////////////////////////////////

/// usage of EXPECT_CALL() for stub functions
// EXPECT_CALL(Stub::getInstance(), stub_func1()).WillOnce(::testing::Return(1));

// MANUAL SECTION: 35d1701a-c77d-57d9-bed5-d756ffe01cc0
// MANUAL SECTION END

/// test fixture for test case
/// local static variables
static uint8_t _controlMotor_pinLeft;
static uint8_t _controlMotor_pinRight;

class Sample : public ::testing::Test {
protected:
  void SetUp() override {
    currDir = Idle;
    timeLeft = 0U;
    lastTimestamp = 0U;
    pinUpdated = 0U;
    _controlMotor_pinLeft = 0U;
    _local_static_vars[std::make_pair("controlMotor", "pinLeft")].second = &_controlMotor_pinLeft;
    _controlMotor_pinRight = 0U;
    _local_static_vars[std::make_pair("controlMotor", "pinRight")].second = &_controlMotor_pinRight;
    // MANUAL SECTION: e8c22ced-203d-5772-b9ea-8237d6edf0f5
    // MANUAL SECTION END
  }
  void TearDown() override {
    // MANUAL SECTION: 773f0158-c9c4-54e6-b79f-f7cff84ce881
    // MANUAL SECTION END
  }
};

/// define a test case for the controlMotor() function
TEST_F(Sample, controlMotor) {
  // MANUAL SECTION: cf139424-4850-5640-aab9-dc3ae9584c7d
  // prepare precondition & expected calls
  ::testing::Sequence seq;
  EXPECT_CALL(Stub::getInstance(), controlPin(testing::Eq(MOTOR_LEFT_PIN), testing::Eq(1))).Times(1).InSequence(seq);
  EXPECT_CALL(Stub::getInstance(), controlPin(testing::Eq(MOTOR_RIGHT_PIN), testing::Eq(1))).Times(1).InSequence(seq);
  EXPECT_CALL(Stub::getInstance(), controlPin(testing::Eq(MOTOR_LEFT_PIN), testing::Eq(0))).Times(1).InSequence(seq);
  EXPECT_CALL(Stub::getInstance(), controlPin(testing::Eq(MOTOR_RIGHT_PIN), testing::Eq(0))).Times(1).InSequence(seq);
  currDir = Forward;
  // call SUT
  controlMotor();
  EXPECT_EQ(LOCAL_STATIC_VARIABLE_READ(controlMotor, pinLeft, uint8_t), 1U);
  // prepare precondition & expected calls
  LOCAL_STATIC_VARIABLE_WRITE(controlMotor, pinLeft, 0);
  currDir = TurnRight;
  // call SUT
  controlMotor();
  EXPECT_EQ(LOCAL_STATIC_VARIABLE_READ(controlMotor, pinLeft, uint8_t), 0U);
  // MANUAL SECTION END
}
/// define a test case for the setDir() function
TEST_F(Sample, setDir) {
  // MANUAL SECTION: c75fc7fa-078f-5ceb-b381-ff2288a65a57
  // MANUAL SECTION END
}
/// define a test case for the move() function
TEST_F(Sample, move) {
  // MANUAL SECTION: afd5313d-1dfd-5f93-8fba-47d757174a4d
  // prepare precondition & expected calls
  ::testing::Sequence seq;
  EXPECT_CALL(Stub::getInstance(), getCurrentTime()).Times(1).InSequence(seq).WillOnce(::testing::Return(10));
  EXPECT_CALL(Stub::getInstance(), controlMotor()).Times(1).InSequence(seq).WillRepeatedly([]() { pinUpdated = 1U; });
  // call SUT
  move(Forward, 10);
  // check result
  EXPECT_EQ(currDir, Forward);
  EXPECT_EQ(timeLeft, 10);
  EXPECT_EQ(lastTimestamp, 10);
  EXPECT_EQ(pinUpdated, 1U);
  // MANUAL SECTION END
}
/// define a test case for the checkTimeout() function
TEST_F(Sample, checkTimeout) {
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
  // check result
  EXPECT_EQ(currDir, Idle);
  // MANUAL SECTION END
}


int main(int argc, char **argv) {
  ::testing::InitGoogleTest(&argc, argv);
  return RUN_ALL_TESTS();
}
