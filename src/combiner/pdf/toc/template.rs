pub const TEMPLATE: &str = r#"
<!DOCTYPE html>
<html lang="en">

<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <style>
        body {
            font-family: SimSun, sans-serif;
            padding: 3%;
        }

        a {
            color: #000000;
            text-decoration: none;
        }

        .item {
            margin-bottom: 5px;
            width: 95%;
        }

        .title {
            background-color: #fff;
        }

        @media print {
            @page {
                size: {{ size }} landscape;
                margin: 0;
            }

            section {
                margin-bottom: 50px;
            }
        }

        .dashed-line-container {
            position: absolute;
            bottom: 10px;
            z-index: -1;
        }

        .dashed-line {
            overflow: hidden;
            width: 90%;
        }

        .page-number {
            position: absolute;
            bottom: 5px;
            right: 10px;
        }

        .break-page {
            page-break-after: always;
        }

        .container {
            position: relative;
            display: flex;
        }
    </style>
</head>

<body>
    <div id="toc">
        <div class="header">
            <div style="margin-bottom: 5px; display: grid; grid-template-columns: auto auto;">
                <div>{{ toc_headers.0 }}</div>
                <div style="text-align: right;">{{ toc_headers.1 }}</div>
            </div>
            <div style="display: grid; grid-template-columns: auto auto;">
                <div>{{ toc_headers.2 }}</div>
                <div style="text-align: right;">{{ toc_headers.3 }}</div>
            </div>
            <div class="item" style="font-weight: bold; font-size: 20px; text-align: center; height: 40px;">{{ content }}
            </div>
        </div>
    </div>
</body>

<script type="text/javascript">
    const data = [
        {% for item in items %}
            {
                id: "{{ item.id }}",
                title: "{{ item.title }}",
                page: "{{ item.page + 1 }}",
            },
        {% endfor %}
    ];

    const toc = document.getElementById("toc");
    let currentPageTitleNumber = 0;
    const newHeader = Array.from(document.getElementsByClassName("header")).pop().cloneNode(true);
    newHeader.style.paddingTop = "4%";
    newHeader.getElementsByClassName("item")[0].innerText = "";


    let detailArea = document.createElement("div");
    detailArea.className = "detail";

    data.forEach((e, index) => {
        if (currentPageTitleNumber > 30) {
            const breaker = document.createElement("div");
            breaker.className = "break-page";
            toc.appendChild(detailArea);
            if (index < data.length) {
                toc.appendChild(breaker);
                toc.appendChild(newHeader.cloneNode(true));
            }
            currentPageTitleNumber = 0;
            detailArea = document.createElement("div");
            detailArea.className = "detail";
        }
        currentPageTitleNumber++;
        const link = document.createElement("a");
        link.id = e.id;
        link.href = '#' + e.id

        const container = document.createElement("div");
        container.className = "container";

        const item = document.createElement("div");
        item.className = "item";
        const title = document.createElement("span");
        title.innerHTML = `${e.title}&#8197`;
        title.style.backgroundColor = "\#ffffff";
        item.appendChild(title);
        container.appendChild(item);

        itemLineContainer = document.createElement("div");
        itemLineContainer.className = "dashed-line-container";
        const itemLine = document.createElement("div");
        itemLine.className = "dashed-line";
        itemLine.innerText = ".".repeat(167);
        itemLineContainer.appendChild(itemLine);
        container.appendChild(itemLineContainer);


        const itemPage = document.createElement("div");
        itemPage.className = "page-number";
        itemPage.innerText = e.page;
        container.appendChild(itemPage);

        link.appendChild(container);
        detailArea.appendChild(link);
    });
    toc.appendChild(detailArea);
</script>

</html>
"#;
