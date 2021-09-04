const urlParams = new URLSearchParams(location.search);
let confirm = document.getElementById("confirm");

confirm.onclick = () => {
  let code = urlParams.get("code");
  location.href = `confirm/real?code=${code}`;
};
