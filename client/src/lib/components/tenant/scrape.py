#!/usr/bin/env python

import json
import time
from concurrent.futures import ThreadPoolExecutor

import numpy as np
import requests
from selenium import webdriver
from selenium.common.exceptions import TimeoutException
from selenium.webdriver.common.by import By
from selenium.webdriver.firefox.options import Options
from selenium.webdriver.support import expected_conditions as EC
from selenium.webdriver.support.ui import WebDriverWait

CHANNELS_LIST_URL = (
    "https://raw.githubusercontent.com/plsuwu/pea-fan/refs/heads/static/channels"
)


def fetch_channel_list() -> list[str]:
    req = requests.get(CHANNELS_LIST_URL)
    content = req.text.splitlines()

    return content


def parse_login(line: str) -> (str, str):
    parts = line.split(":")
    print(parts)

    return (parts[0], parts[1])


def parse_link_domain(link: str, title: str):
    domain = link.split("https://")[1]
    if domain[:4] == "www.":
        domain = domain[4:]

    domain = domain.split(".")[0]
    return {"name": domain, "url": link, "title": title}


def new_webdriver():
    opts = Options()
    opts.add_argument("--headless")

    opts.page_load_strategy = "eager"
    return webdriver.Firefox(options=opts)


def fetch_links(login: str):
    driver = new_webdriver()

    url = f"https://twitch.tv/{login}/about"
    links = {"name": login, "links": []}

    print(f"running selenium GET for '{url}'")
    driver.get(url)

    try:
        ctr_elements = WebDriverWait(driver, timeout=3, poll_frequency=0.5).until(
            EC.visibility_of_all_elements_located(
                (By.CSS_SELECTOR, ".social-media-link__container")
            )
        )

        anchors = [(el.find_element(By.TAG_NAME, "a"), el.text) for el in ctr_elements]
        links["links"] = [
            parse_link_domain(a[0].get_attribute("href"), a[1]) for a in anchors
        ]

        driver.quit()

        print(f"{login} -> found:", [link["name"] for link in links["links"]])
        # return links

    except TimeoutException:
        print(f"{login} -> request timed out")
        driver.quit()

    except Exception as e:
        print(f"{login} -> unknown error:", e)
        driver.quit()

    finally:
        print("momentarily pausing...")
        time.sleep(1)

        return links


def process_chunk(chunk: list[str]):
    results = []

    for channel in chunk:
        (login, id) = parse_login(channel)
        links = fetch_links(login)

        results.append({"id": id, "data": links})

    return results


if __name__ == "__main__":
    THREAD_WORKERS = 20

    channels = fetch_channel_list()
    chunks = np.array_split(channels, 5)
    output = []

    with ThreadPoolExecutor(max_workers=THREAD_WORKERS) as exec:
        results = list(exec.map(process_chunk, chunks))
        output.append(results)

    print(output)

    with open("links.json", "a") as outfile:
        json.dump(output, outfile, indent=2)
