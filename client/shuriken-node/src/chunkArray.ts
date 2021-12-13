export const chunkArray = <T>(array: T[], size: number): T[][] => {
    const chunked_arr = [];
    const copied = [...array]; // ES6 destructuring
    const numOfChild = Math.ceil(copied.length / size); // Round up to the nearest integer
    for (let i = 0; i < numOfChild; i++) {
        chunked_arr.push(copied.splice(0, size));
    }
    return chunked_arr;
};
