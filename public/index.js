// Example POST method implementation:
async function postData(url = "", data = {}) {
    // Default options are marked with *
    const response = await fetch(url, {
        method: "POST", // *GET, POST, PUT, DELETE, etc.
        mode: "cors", // no-cors, *cors, same-origin
        cache: "no-cache", // *default, no-cache, reload, force-cache, only-if-cached
        credentials: "same-origin", // include, *same-origin, omit
        headers: {
        "Content-Type": "application/json",
        // 'Content-Type': 'application/x-www-form-urlencoded',
        },
        redirect: "follow", // manual, *follow, error
        referrerPolicy: "no-referrer", // no-referrer, *no-referrer-when-downgrade, origin, origin-when-cross-origin, same-origin, strict-origin, strict-origin-when-cross-origin, unsafe-url
        body: JSON.stringify(data), // body data type must match "Content-Type" header
    });

    return response.json(); // parses JSON response into native JavaScript objects
}

const rust_src = `
fn main() {
    let param = std::env::var("param").unwrap();

    // Print text to the console.
    println!("Hello World, param = {}", param);
}
`;

postData("/api/manual", { name: 'Hello, Post to /api/manual !', number: 42, data_b64: btoa(rust_src) }).then((data) => {
    console.log(data); // JSON data parsed by `data.json()` call
});
postData("/api/mjsonrust", { name: 'Hello, Post to /api/mjsonrust !', number: 42 }).then((data) => {
    console.log(data); // JSON data parsed by `data.json()` call
});