const urlParams = new URLSearchParams(location.search);
let confirm = document.getElementById("confirm");

confirm.onclick = () => {
  let code = urlParams.get("code");
  location.href = `/subscribe/confirm/real?code=${code}`;
};
