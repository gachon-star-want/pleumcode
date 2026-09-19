# PleumRouter만의 독창적 기능

> 상태: **v1 범위 확정됨(2026-09-19 인터뷰).** 아래는 확정된 기능 + 근거.

기존 오픈소스 코딩 에이전트(Aider/Goose/opencode/OpenHands 등)는 전부 "텍스트 tool-use 루프"에 머문다.
PleumRouter가 서버사이드에 이미 갖고 있고 다른 어떤 코딩 에이전트도 흉내 낼 수 없는 것들 위주로 골랐다.

## 핵심 통찰: 하나의 엔진, 네 개의 프리셋

인터뷰에서 사용자가 강조한 요구("우리 같은 router 서비스의 핵심은 서브에이전트마다 다른 모델을 쓰는 것 —
무조건 지원해야 함")를 조사하다가 중요한 사실을 확인했다: **Goose는 이미 subagent/subrecipe 아키텍처를
네이티브로 갖고 있고, 서브에이전트/subrecipe마다 다른 모델·provider를 지정하는 기능도 이미 존재한다**
(`recipe.settings.goose_model`/`goose_provider` 필드, 기본값은 `GOOSE_SUBAGENT_MODEL`/`GOOSE_SUBAGENT_PROVIDER`
env — 단, 둘 다 아직 "experimental" 상태). 즉 이것이 fork의 핵심 확장 포인트가 되어야 한다.

여기서 나온 설계 원칙: **선택된 4개 기능 + 신규 요구사항을 각각 따로 만들지 않고, "역할(role)별로 다른
PleumRouter 모델을 배정하는 멀티모델 서브에이전트 오케스트레이션 엔진" 하나를 코어로 만들고, 나머지는 그
위에 얹는 프리셋/UX로 설계한다.**

- Cascade 검증 하네스 = 순차 프리셋(초안 subagent 저가 모델 → 검증 subagent 고가 모델, 실패 시 승격)
- Fusion 병렬 구현/리뷰 = 병렬 프리셋(같은 작업을 N개 subagent에 다른 모델로 동시 위임 후 합성)
- Smart Mode = 자동 프리셋(role별 기본 모델을 PleumRouter가 의도기반으로 자동 채워줌)

## 확정 기능 (v1)

| # | 기능 | 설명 | 왜 우리만 가능한가 |
|---|---|---|---|
| 1 | **멀티모델 서브에이전트 오케스트레이션** (신규, 최우선) | 코드 탐색/구현/테스트작성/리뷰 등 역할(role)마다 다른 모델·provider를 명시적으로 배정. Goose의 native subagent/subrecipe에 `settings.goose_model`을 role별로 채워 넣는 방식 | PleumRouter의 마켓플레이스(1모델=N프로바이더)·라우팅정책 인프라 덕분에 "역할별 최적 모델 프리셋"을 서버가 추천/관리할 수 있음 |
| 2 | 멀티모달 tool 통합 | 코딩 중 아이콘/아키텍처 다이어그램(이미지)·코드 설명 음성 브리핑(TTS)·데모 영상까지 에이전트가 직접 호출 | 텍스트·임베딩·이미지·오디오·영상을 **동일 크레딧 원장**으로 과금하는 게이트웨이는 PleumRouter뿐 |
| 3 | Fusion 병렬 구현/리뷰 모드 | 어려운 버그/설계에서 2~4개 모델을 병렬 실행해 답을 종합 | PleumRouter의 Fusion(`/v1/fusion`)을 #1 엔진의 병렬 프리셋으로 노출 |
| 4 | Cascade 검증 하네스 | 저가 모델 초안 → 테스트/타입체크 통과할 때까지 고가 모델로 자동 승격 | routing-policies의 cascade 오케스트레이터를 #1 엔진의 순차 프리셋으로 재사용. "체계적으로 잘하는" 하네스의 핵심 |
| 5 | KRW 실시간 비용 표시 + 예산 가드레일 | 턴/세션별 실시간 원화 비용 스트림, 예산 초과 시 경고·일시정지. 서브에이전트별로도 개별 비용 분해 표시 | KRW 선불크레딧 원장이 이미 있어 다른 에이전트는 흉내내려면 별도 과금 연동이 필요 |
| 6 | Smart Mode 자동 배정 (보너스, 저비용) | role별 기본 모델을 직접 안 고르면 PleumRouter가 의도기반으로 자동 채움 | 서버사이드에 이미 구현돼 있어 모델 ID 패스스루만으로 거의 공짜로 딸려옴 |

## v2 이후 후보 (이번 v1엔 미포함)

- 한국어 네이티브 UX + PIPA 가드레일(커밋메시지 한국어화, 코드 내 개인정보 하드코딩 감지) — 필요성은 있으나
  구현 범위가 커서 v1 이후로 미룸.

## 구현 시 반드시 챙길 기술 리스크

- **알려진 업스트림 버그(2026-09, `aaif-goose/goose#11862`)**: `GOOSE_SUBAGENT_MODEL` 환경변수를 전역으로
  설정하면 recipe별로 지정한 role-specific 모델이 전부 하나로 collapse되는 버그가 있음. → **우리 구현은
  절대 전역 env var에 의존하지 말고, 매 subagent/subrecipe 호출마다 명시적으로 `settings.goose_model`을
  주입하는 방식으로만 설계**(fork 시점에 이 버그가 안 고쳐졌으면 우리 fork에서 먼저 패치).
- subagent/subrecipe 둘 다 goose 자체에서 "experimental" 상태 — API가 바뀔 수 있음을 감안해 우리 쪽
  추상화 레이어(role→model 매핑)를 얇게 유지해서 upstream 변경에 흔들리지 않게 할 것.
- Goose subagent는 재귀 생성 불가(subagent가 또 subagent를 못 만듦) — role 계층을 깊게 설계하면 안 됨,
  1단계 위임까지만.
