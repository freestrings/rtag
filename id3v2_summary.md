# ID3v2 Overview
2.4
```
+-----------------------------+
|      Header (10 bytes)      |
+-----------------------------+
|       Extended Header       |
| (variable length, OPTIONAL) |
+-----------------------------+
|   Frames (variable length)  |
+-----------------------------+
|           Padding           |
| (variable length, OPTIONAL) |
+-----------------------------+
| Footer (10 bytes, OPTIONAL) |
+-----------------------------+
```

## Id3v2 2.4 footer
- 푸터는 헤더의 카피. 식별자만 "3DI"

# ID3v2 Header
- 파일 제일 처음에 위치
- 10 bytes
- 49 44 33 yy yy xx zz zz zz zz 패턴
    yy는 0xFF보다 작다(254)
    xx는 플래그
    zz는 0x80보다 작다(127)
```
# 0~2: ID3 (상수로 고정된 값으로 ID3v2임을 나타냄)
# 3: major 버전
# 4: revision 넘버
# 5: ID3v2 flags (abc0 0000)
    a:Unsynchronisation shema: 100x xxxx
    b: Extended header: 010x xxxx
    c: Experimental indicator: 001x xxxx
# 6~9: 전체 태크 크기. 패딩은 포함되고 헤더는 뺀. 그렇지만 extened header는 포함됨 => 전체 태그크기 - 10
    - 4 byte
    - 각 바이트의 최상위 비트는 0. 총 28개 비트
    - 0 비트는 뺀 257 바이트 태그 길이는 00 00 02 01로 표현됨. ??
```


# ID3v2 extended header
TODO

# ID3v2 frame
- 태그는 헤더와 바디로 구성
- 바디는 `하나 이상`의 프레임으로 구성
- 프레임은 프레임 헤더와 `하나 이상`의 필드로 구성
```
태그 = 헤더 + (바디 = 1* x (프레임 = 1* x 필드)).
```

## Frame header (10 bytes)
2.3
```
0~3: 프레임 ID. A-Z, 0-9. "X", "Y", "Z"로 시작되면 실험적 목적
4~7: 프레임 크기. 프레임 크기에서 프레임 헤더 크기인 10 바이트는 제외
8~9: 플래그 (abc0 0000 ijk0 0000)
    8: 상태 메시지
        a: 태그 변경 보존: axxx xxxx (0: 프레임 보존, 1: 프레임 버림)
        b: 파일 변경 보존: xbxx xxxx (0: 파일 보존, 1: 파일 버림)
        c: 읽기 전용: xxcx xxxx
    9: 인코딩
        i: 프레임 압축여부: xxxx ixxx 
            0: 압축안됨 
            1: zlib사용. 4바이트로 압축됨. 프레임 헤더뒤에 붙음
        j: 프레임 암호화여부: xxxx xjxxx
            0: 암호화 안됨 
            1: 암호화됨
        k: 그룹 식별자 정보 포함 여부: xxx xxkx
            0: 그룹정보 포함하지 않음 
            1: 그룹정보 포함함
               그룹 식별자 바이트가 프레임 헤더에 추가됨 
               동일한 그룹 식별자를 가진 모든 프레임은 동일한 그룹
```

2.4
```
8~9: 플래그 (0abc0000 0h00kmnp)
    8: 상태 메시지
        a: 태그 변경 보존: xaxx xxxx (0: 프레임 보존, 1: 프레임 버림)
        b: 파일 변경 보존: xxbx xxxx (0: 파일 보존, 1: 파일 버림)
        c: 읽기 전용: xxxc xxxx
    9: 인코딩
        h: 그룹 식별자 정보 포함 여부 xhxxx xxx
            0: 그룹정보 포함하지 않음 
            1: 그룹정보 포함함
        k: 프레임 압축여부: xxxx kxxx
            0: 압축 안됨
            1: zlib로 압축됨. `Data Length Indicator` 비트가 잘 입력 되어야 함
        m: 프레임 암호화 여부: xxxx xmxx
            0: 프레임 암호화 안됨
            1: 프레임 암호화 됨
                "ENCR" 프레임에 관련 정보가 있음
                압축작업 뒤에 암호화 되어야 함
                알고리즘에 따라 `Data Length Indicator`가 있어야 함
        n: 프레임에 Unsynchronisation 적용 여부
            0: Unsynchronised 프레임
            1: Synchronised 프레임
        p: Data Length Indicator: xxxx xxxp
            0: 데이터 길이 지시자가 없음
            1: 데이터 길이 지시자 있음
                프레임 끝에 데이터 길이 지시자가 추가 되어있다
```

- 프레임 순서는 없음
- 프레임은 최소 하나의 프레임을 포함
- 프레임은 헤더를 제외하고 최소 1바이트 크기
- 기본 ISO-8859-1로 표현된 0x20~0xFF 사이의 문자열
- 유니코드 문자열은 16비트 유니코드 2.0을 사용
- 유니코드 문자열은 반드시 유니코드 BOM(0xFFFE 또는 0xFEFF)으로 시작해야 함
- 숫자로된 문자열이나 URL은 ISO-8859-1로 인코딩
- ISO-8859-1로 인코딩 되면 종료문자는 0x00로 끝남. 기본 newline 숨김. 0x0A로 끝나면 newline 허용. 
- 유니코드로 인코딩 되면 종료 문자는 0x0000으로 끝남
- 프레임 크기 뒤에 인코딩 바이트가 있을 수 있다.
2.3
```
    0x00: ISO-8859-1[0x00], ISO-8859-1[0x0A]=>newline 허용
    0x01: [BOM]UTF-16[0x0000]
```
2.4
```
    0x00: ISO-8859-1[0x00]
    0x01: [BOM]UTF-16[0x0000]
    0x02: UTF-16BE[0x0000]
    0x03: UTF-8[0x00] => UTF-8 인코딩된 유니코드
```

- 빈 유니코드 문자열도 유니코드 BOM문자 뒤에 NULL 문자가 와야 한다.
```
    0xFFFE0000
    0xFEFF0000
```

# 2.4 Tag location
- 기본 태그 위치는 오디오 파일 맨 앞에 위치한다  
- 뒤쪽에 위치할 수 도 있다
- 앞쪽 뒤쪽 혼합도 가능하다
```
    앞쪽에 중요한 정보를 담은 첫번째 태그를 둔다 => `SEEK frame`
    두번째 태그는 파일 뒤에 붙인다
```

# Declared ID3v2 frames
TODO

## Text information frames
- 텍스트 정보 프레임은 Artist, Album과 같은 정보를 포함하고 있다 
- 태그에서 (보통) 유일한 프레임이다
- 0x00(00)이 뒤에 붙으면 정보는 무시되고 표시되지 않는다
- 프레임 식별자에 "XXX"는 사용자 정의 프레임 (예, TXXX: 사용자 정의 텍스트 프레임)
- "T???" 텍스트 프레임 식별자
```
    [T000~TZZZ][$xx: 인코딩 정보][텍스트]
```
- "COMM" 코멘트 프레임 식별자
```
    [COMM][$xx: 인코딩][$xx xx xx: 언어][짧은설명$00(00)][실제 텍스트]
```
- "APIC" 첨부된 그림. 
    복수 이미지가 첨부 될 수 있음
    MIME type -->URL 형식으로 이미지 링크를 넣을 수 있음
```
    [APIC][$xx][MIME type$00][$xx: Picture type][텍스트$00(00)][이미지 바이너리]
```


http://id3.org/id3v2.3.0
http://id3.org/id3v2.4.0