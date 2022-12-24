import {
  component$,
  useClientEffect$,
  useStore,
  useStylesScoped$,
} from "@builder.io/qwik";
import styles from "./book_viewer.css?inline";

import { generate_image } from "~/../book-wasm/pkg/book_wasm"

interface font {
  font: string;
}

export default component$((props: { input: string }) => {
  useStylesScoped$(styles);

  const font = useStore<font>(() => ({ font: "" }));

  useClientEffect$(async () => {
    const res = await fetch("/default.json");
    font.font = await res.text();
  });


  useClientEffect$(async ({ track }) => {
    track(() => props.input);

    if (font.font === "") return;

    const image_bytes = generate_image(font.font, props.input);
    const canvas = (
      document.getElementById("book-img") as HTMLCanvasElement
    ).getContext("2d");

    const img = new Image();
    img.onload = () => {
      canvas?.drawImage(img, 0, 0);
    };
    const blob = URL.createObjectURL(new Blob([image_bytes]));
    img.src = blob;
  });

  return (
    <div class="input-box">
      <canvas id="book-img" width={146} height={180}></canvas>
    </div>
  );
});
