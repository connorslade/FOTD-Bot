let page = 0;
let end = false;

const dateString = (date) =>
  `${date.getFullYear()}-${(1 + date.getMonth())
    .toString()
    .padStart(2, "0")}-${date.getDate().toString().padStart(2, "0")}`;

const getFacts = async (page) => {
  const URI = `/api/fact/history?page=${page}`;
  const rawResponse = await fetch(URI);
  const json = await rawResponse.json();
  end = json.end || end;

  const content = document.querySelector(".content");
  json.facts.forEach((fact) => {
    const date = new Date((1 + fact.date) * (60 * 60 * 24 * 1000));
    const fotd = document.createElement("div");
    fotd.classList.add("history-quote");
    fotd.addEventListener(
      "click",
      () => (window.location = `/fact?day=${fact.date}`)
    );

    fotd.innerHTML = `
        <p class="history-date">${dateString(date)}</p>
        <p class="history-fact">${fact.fact}</p>
        `;

    content.appendChild(fotd);
  });
};

getFacts(0);
setInterval(() => {
  const { scrollTop, scrollHeight, clientHeight } = document.documentElement;
  if (scrollTop + clientHeight >= scrollHeight * 0.8 && !end) getFacts(++page);
  if (end) document.querySelector(".end").classList.remove("hidden");
}, 100);
