from bs4 import BeautifulSoup
import requests

url = 'https://www.unicode.org/charts/nameslist/n_FF00.html'
response = requests.get(url)
soup = BeautifulSoup(response.text, 'html.parser')

table_elements = soup.find('table').find_all('tr')

h_to_f = []

tmp = []

for row in table_elements:
    cells = row.find_all('td')
    if len(cells) == 3:
        # print(cells[0].text, cells[1].text, cells[2].text)
        # output) FF01  ！  Fullwidth Exclamation Mark

        # get next line
        tmp = ['', cells[1].text.replace(u'\xa0', '')]
    elif len(cells) == 4 and cells[2].text == '≈':
        tmp[0] = cells[3].text[-1]
        h_to_f.append(tmp)

for h, f in h_to_f:
    print(f'("{h}", "{f}"),')