# Results!

## Rust
Average server response time is 240ms, varying from 190ms to 400ms. Efforts were concentrated into making the code run as fast as possible.
- Used of a better algorithm to resolve the calculations, but did not win against TypeScript in terms of response time and execution time, which are not intended.
- The most aggresive code optimizations for the binary were incremented, but the result did not go as expected.

## TypeScript
Average server response time of 90ms, and sometimes can reach 60ms. Used of worse algorithm and still won in terms of performance against the following code in Rust.

# Notes
- This was my first attempt using Rust in a larger scale, I still need to look up for more possible optimizations and techniques to make the code run better. The result of 266% slower should not be this high, I was expecting something in range of 50% maximum.

- Before the addition of Sea-ORM and Actix web server, Rust was presenting great results, with average response time for hardcoded JSON's in the fn calculate() presenting 50% better results than TypeScript, later drastically reduced with the addition of these libraries. It could process 5 requests in 300ms, while TypeScript took an average of 1.1 seconds, meaning that by that time Rust was approximately 350% faster than TypeScript.

# Predictions

- Probably the addition of Rust-Actix and Sea-ORM are not well implemented and not performance-optimized, causing this program to be slower than the one made in TypeScript since considering only the algorithm and the fn calculate(), Rust was considerably faster than the one in TypeScript.