#include "pico/stdlib.h"
#include "hardware/pwm.h"

int main() {
    for (int i = 0; i < 30; i += 2) { 
        gpio_set_function(i + 0, GPIO_FUNC_PWM);
        gpio_set_function(i + 1, GPIO_FUNC_PWM);
        uint slice_num = pwm_gpio_to_slice_num(i);
        pwm_set_chan_level(slice_num, PWM_CHAN_A, 1);
        pwm_set_chan_level(slice_num, PWM_CHAN_B, 3);
    }

    pwm_set_mask_enabled(0xFFFFFFFF);
}
