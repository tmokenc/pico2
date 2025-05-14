unsigned int factorial(unsigned n) {
    if (n == 0) {
        return 1; // Base case: 0! = 1
    }
    return n * factorial(n - 1); // Recursive case
}

int main() {
    unsigned int number = 10; // Change this value to calculate a different factorial

    // Calculate the factorial
    // volatile here is used to prevent the compiler from optimizing away the function call
    volatile unsigned int factorial_result = factorial(number);
	__asm__ volatile ("ebreak");
	// The result should be in the x15 (or a5) register of the processor

    return 0;
}
