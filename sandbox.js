(() => {
  const boxes = document.querySelectorAll("details.admonish-sandbox");
  boxes.forEach((box) => {
    const once = () => {
      const template = box.querySelector("template");
      const sandbox = template.content.cloneNode(true);
      template.after(sandbox);

      box.removeEventListener("toggle", once);
    };
    box.addEventListener("toggle", once);
  });
})();
