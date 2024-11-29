import time
import requests
import re
import hashlib

password = "password"

headers = {
    'Accept': '*/*',
    'Accept-Language': 'en-GB,en',
    'Connection': 'keep-alive',
    'Referer': 'http://192.168.11.1/',
    'Sec-GPC': '1',
    'User-Agent': 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/114.0.0.0 Safari/537.36',
    'X-Requested-With': 'XMLHttpRequest',
}

s = requests.Session()


def get_salt_key():
  g = s.get("http://192.168.11.1/?_type=loginData&_tag=login_token",
            headers=headers, verify=False)
  salt_key = re.findall(
      r"<ajax_response_xml_root>(.*)</ajax_response_xml_root>", g.text)[0]
  return salt_key


def sha256(s):
  return hashlib.sha256(s.encode('utf-8')).hexdigest()


def login():
  s.get("http://192.168.11.1/", headers=headers, verify=False)
  data = {
      "action": "login",
      "Username": "user",
      "Password": sha256(password + get_salt_key())
  }
  s.post("http://192.168.11.1/?_type=loginData&_tag=login_entry",
         data=data, headers=headers, verify=False)


def Bps_to_Mbps(Bps: str) -> str:
  Mbps = int(Bps) / 8388608  # 8 * 1024 * 1024
  return f"{Mbps:.2f}"


def print_data(data):
  for key, device in data['ad'].items():
    if key == 'MGET_INST_NUM':
      continue
    print(f"{device['HostName']:30} {Bps_to_Mbps(device['TxRateBps']):5} {Bps_to_Mbps(device['RxRateBps']):5} {device['IpAddr']:15} {device['MacAddr']}")
  print("-" * 70)


def main():
  login()
  s.get('http://192.168.11.1/?_type=menuView&_tag=mmTopology&Menu3Location=0',
        headers=headers, verify=False)
  while True:
    try:
      data = s.get('http://192.168.11.1/?_type=menuData&_tag=topo_lua.lua',
                   headers=headers, verify=False).json()
      print_data(data)
      time.sleep(1)
    except KeyboardInterrupt:
      break


if __name__ == "__main__":
  main()
