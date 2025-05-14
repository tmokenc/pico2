#include "pico/stdlib.h"
#include "hardware/timer.h"

#define WAIT_TIME 1000000 // 1 second with 1 MHz clock

int main() {
    busy_wait_us_32(WAIT_TIME);
}
