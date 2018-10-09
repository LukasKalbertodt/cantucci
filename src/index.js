const { greet } = wasm_bindgen;


wasm_bindgen('./cantucci_bg.wasm').then(x => {
    console.log('yeah');
    greet('peter');
}).catch(console.error);
