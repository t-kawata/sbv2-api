import requests


data = (requests.get("http://localhost:8080/audio_query", params={
    "text": "こんにちは、今日はいい天気ですね。",
})).json()
print(data)

data = (requests.post("http://localhost:8080/synthesis", json={
    "text": data["text"],
    "ident": "tsukuyomi",
    "speaker_id": 0,
    "style_id": 0,
    "sdp_ratio": 0.5,
    "length_scale": 0.5,
    "audio_query": data["audio_query"],
})).content
with open("test.wav", "wb") as f:
    f.write(data)