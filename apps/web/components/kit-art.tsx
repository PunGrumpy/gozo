/* the kit line art: ink is currentColor and paper is the inherited fill, so it follows the theme */
import type { ReactNode } from "react";

const boxView = "-106.5 -57 247.6 193";

export const art = {
  "brick-build": {
    body: (
      <g>
        <polygon
          points="17.3,61.0 82.3,98.5 82.3,62.5 17.3,25.0"
          stroke="currentColor"
          strokeWidth="1.25"
          strokeLinejoin="round"
        />
        <polygon
          points="129.9,71.0 82.3,98.5 82.3,62.5 129.9,35.0"
          stroke="currentColor"
          strokeWidth="1.25"
          strokeLinejoin="round"
        />
        <polygon
          points="65.0,-2.5 129.9,35.0 82.3,62.5 17.3,25.0"
          stroke="currentColor"
          strokeWidth="1.25"
          strokeLinejoin="round"
        />
        <line
          x1="119.5"
          y1="51.0"
          x2="119.5"
          y2="69.0"
          stroke="currentColor"
          strokeWidth="1.25"
          strokeLinecap="round"
        />
        <line
          x1="115.2"
          y1="53.5"
          x2="115.2"
          y2="66.5"
          stroke="currentColor"
          strokeWidth="1.25"
          strokeLinecap="round"
        />
        <line
          x1="110.9"
          y1="56.0"
          x2="110.9"
          y2="64.0"
          stroke="currentColor"
          strokeWidth="1.25"
          strokeLinecap="round"
        />
        <line
          x1="106.5"
          y1="58.5"
          x2="106.5"
          y2="76.5"
          stroke="currentColor"
          strokeWidth="1.25"
          strokeLinecap="round"
        />
        <line
          x1="102.2"
          y1="61.0"
          x2="102.2"
          y2="74.0"
          stroke="currentColor"
          strokeWidth="1.25"
          strokeLinecap="round"
        />
        <line
          x1="97.9"
          y1="63.5"
          x2="97.9"
          y2="71.5"
          stroke="currentColor"
          strokeWidth="1.25"
          strokeLinecap="round"
        />
        <line
          x1="93.5"
          y1="66.0"
          x2="93.5"
          y2="84.0"
          stroke="currentColor"
          strokeWidth="1.25"
          strokeLinecap="round"
        />
        <line
          x1="89.2"
          y1="68.5"
          x2="89.2"
          y2="81.5"
          stroke="currentColor"
          strokeWidth="1.25"
          strokeLinecap="round"
        />
      </g>
    ),
    viewBox: boxView,
  },
  "brick-deploy": {
    body: (
      <g>
        <polygon
          points="-30.3,88.5 34.6,126.0 34.6,90.0 -30.3,52.5"
          stroke="currentColor"
          strokeWidth="1.25"
          strokeLinejoin="round"
        />
        <polygon
          points="82.3,98.5 34.6,126.0 34.6,90.0 82.3,62.5"
          stroke="currentColor"
          strokeWidth="1.25"
          strokeLinejoin="round"
        />
        <polygon
          points="17.3,25.0 82.3,62.5 34.6,90.0 -30.3,52.5"
          stroke="currentColor"
          strokeWidth="1.25"
          strokeLinejoin="round"
        />
        <ellipse cx="-28.6" cy="61.5" rx="1.51" ry="1.26" fill="currentColor" />
        <ellipse cx="-24.9" cy="63.6" rx="1.51" ry="1.26" fill="currentColor" />
        <ellipse cx="-28.6" cy="65.7" rx="1.51" ry="1.26" fill="currentColor" />
        <ellipse cx="-28.6" cy="78.3" rx="1.51" ry="1.26" fill="currentColor" />
        <ellipse cx="-24.9" cy="80.4" rx="1.51" ry="1.26" fill="currentColor" />
        <ellipse cx="-14.0" cy="69.9" rx="1.51" ry="1.26" fill="currentColor" />
        <ellipse cx="-10.4" cy="72.0" rx="1.51" ry="1.26" fill="currentColor" />
        <ellipse cx="-6.8" cy="74.1" rx="1.51" ry="1.26" fill="currentColor" />
        <ellipse cx="-17.7" cy="72.0" rx="1.51" ry="1.26" fill="currentColor" />
        <ellipse cx="-3.1" cy="80.4" rx="1.51" ry="1.26" fill="currentColor" />
        <ellipse cx="-17.7" cy="76.2" rx="1.51" ry="1.26" fill="currentColor" />
        <ellipse cx="-3.1" cy="84.6" rx="1.51" ry="1.26" fill="currentColor" />
        <ellipse cx="-17.7" cy="80.4" rx="1.51" ry="1.26" fill="currentColor" />
        <ellipse cx="-3.1" cy="88.8" rx="1.51" ry="1.26" fill="currentColor" />
        <ellipse cx="-14.0" cy="86.7" rx="1.51" ry="1.26" fill="currentColor" />
        <ellipse cx="-10.4" cy="88.8" rx="1.51" ry="1.26" fill="currentColor" />
        <ellipse cx="-6.8" cy="90.9" rx="1.51" ry="1.26" fill="currentColor" />
      </g>
    ),
    viewBox: boxView,
  },
  "brick-dev": {
    body: (
      <g>
        <polygon
          points="-47.6,23.5 17.3,61.0 17.3,25.0 -47.6,-12.5"
          stroke="currentColor"
          strokeWidth="1.25"
          strokeLinejoin="round"
        />
        <polygon
          points="65.0,33.5 17.3,61.0 17.3,25.0 65.0,-2.5"
          stroke="currentColor"
          strokeWidth="1.25"
          strokeLinejoin="round"
        />
        <polygon
          points="0.0,-40.0 65.0,-2.5 17.3,25.0 -47.6,-12.5"
          stroke="currentColor"
          strokeWidth="1.25"
          strokeLinejoin="round"
        />
      </g>
    ),
    viewBox: boxView,
  },
  "brick-env": {
    body: (
      <g>
        <polygon
          points="-95.3,51.0 -30.3,88.5 -30.3,52.5 -95.3,15.0"
          stroke="currentColor"
          strokeWidth="1.25"
          strokeLinejoin="round"
        />
        <polygon
          points="17.3,61.0 -30.3,88.5 -30.3,52.5 17.3,25.0"
          stroke="currentColor"
          strokeWidth="1.25"
          strokeLinejoin="round"
        />
        <polygon
          points="-47.6,-12.5 17.3,25.0 -30.3,52.5 -95.3,15.0"
          stroke="currentColor"
          strokeWidth="1.25"
          strokeLinejoin="round"
        />
        <ellipse cx="-79.5" cy="32.1" rx="1.51" ry="1.26" fill="currentColor" />
        <ellipse cx="-75.9" cy="34.2" rx="1.51" ry="1.26" fill="currentColor" />
        <ellipse cx="-72.2" cy="36.3" rx="1.51" ry="1.26" fill="currentColor" />
        <ellipse cx="-83.1" cy="34.2" rx="1.51" ry="1.26" fill="currentColor" />
        <ellipse cx="-83.1" cy="38.4" rx="1.51" ry="1.26" fill="currentColor" />
        <ellipse cx="-72.2" cy="44.7" rx="1.51" ry="1.26" fill="currentColor" />
        <ellipse cx="-68.6" cy="46.8" rx="1.51" ry="1.26" fill="currentColor" />
        <ellipse cx="-83.1" cy="42.6" rx="1.51" ry="1.26" fill="currentColor" />
        <ellipse cx="-68.6" cy="51.0" rx="1.51" ry="1.26" fill="currentColor" />
        <ellipse cx="-79.5" cy="48.9" rx="1.51" ry="1.26" fill="currentColor" />
        <ellipse cx="-75.9" cy="51.0" rx="1.51" ry="1.26" fill="currentColor" />
        <ellipse cx="-72.2" cy="53.1" rx="1.51" ry="1.26" fill="currentColor" />
        <ellipse cx="-57.7" cy="44.7" rx="1.51" ry="1.26" fill="currentColor" />
        <ellipse cx="-54.0" cy="46.8" rx="1.51" ry="1.26" fill="currentColor" />
        <ellipse cx="-50.4" cy="48.9" rx="1.51" ry="1.26" fill="currentColor" />
        <ellipse cx="-61.3" cy="46.8" rx="1.51" ry="1.26" fill="currentColor" />
        <ellipse cx="-46.8" cy="55.2" rx="1.51" ry="1.26" fill="currentColor" />
        <ellipse cx="-61.3" cy="51.0" rx="1.51" ry="1.26" fill="currentColor" />
        <ellipse cx="-46.8" cy="59.4" rx="1.51" ry="1.26" fill="currentColor" />
        <ellipse cx="-61.3" cy="55.2" rx="1.51" ry="1.26" fill="currentColor" />
        <ellipse cx="-46.8" cy="63.6" rx="1.51" ry="1.26" fill="currentColor" />
        <ellipse cx="-57.7" cy="61.5" rx="1.51" ry="1.26" fill="currentColor" />
        <ellipse cx="-54.0" cy="63.6" rx="1.51" ry="1.26" fill="currentColor" />
        <ellipse cx="-50.4" cy="65.7" rx="1.51" ry="1.26" fill="currentColor" />
        <ellipse cx="-39.5" cy="55.2" rx="1.51" ry="1.26" fill="currentColor" />
        <ellipse cx="-35.9" cy="57.3" rx="1.51" ry="1.26" fill="currentColor" />
        <ellipse cx="-32.2" cy="59.4" rx="1.51" ry="1.26" fill="currentColor" />
        <ellipse cx="-32.2" cy="67.8" rx="1.51" ry="1.26" fill="currentColor" />
        <ellipse cx="-35.9" cy="69.9" rx="1.51" ry="1.26" fill="currentColor" />
        <ellipse cx="-39.5" cy="72.0" rx="1.51" ry="1.26" fill="currentColor" />
        <ellipse cx="-35.9" cy="74.1" rx="1.51" ry="1.26" fill="currentColor" />
        <ellipse cx="-32.2" cy="76.2" rx="1.51" ry="1.26" fill="currentColor" />
      </g>
    ),
    viewBox: boxView,
  },
  "brick-gomod": {
    body: (
      <g>
        <polygon
          points="-95.3,55.0 34.6,130.0 34.6,126.0 -95.3,51.0"
          stroke="currentColor"
          strokeWidth="1.25"
          strokeLinejoin="round"
        />
        <polygon
          points="129.9,75.0 34.6,130.0 34.6,126.0 129.9,71.0"
          stroke="currentColor"
          strokeWidth="1.25"
          strokeLinejoin="round"
        />
        <polygon
          points="0.0,-4.0 129.9,71.0 34.6,126.0 -95.3,51.0"
          stroke="currentColor"
          strokeWidth="1.25"
          strokeLinejoin="round"
        />
      </g>
    ),
    viewBox: boxView,
  },
  "brick-gozo": {
    body: (
      <g>
        <g transform="translate(17.3 17.0) scale(0.9) rotate(15)">
          <path
            d="M0 0 C 18 -14 20 -40 0 -44 C -20 -40 -18 -14 0 0 Z"
            transform="rotate(-90) translate(0 -3)"
            fill="currentColor"
          />
          <path
            d="M0 0 C 18 -14 20 -40 0 -44 C -20 -40 -18 -14 0 0 Z"
            transform="rotate(30) translate(0 -3)"
            fill="currentColor"
          />
          <path
            d="M0 0 C 18 -14 20 -40 0 -44 C -20 -40 -18 -14 0 0 Z"
            transform="rotate(150) translate(0 -3)"
            fill="currentColor"
          />
        </g>
      </g>
    ),
    viewBox: boxView,
  },
  "brick-status": {
    body: (
      <g>
        <polygon
          points="-100.5,15.0 34.6,93.0 34.6,85.0 -100.5,7.0"
          stroke="currentColor"
          strokeWidth="1.25"
          strokeLinejoin="round"
        />
        <polygon
          points="135.1,35.0 34.6,93.0 34.6,85.0 135.1,27.0"
          stroke="currentColor"
          strokeWidth="1.25"
          strokeLinejoin="round"
        />
        <polygon
          points="0.0,-51.0 135.1,27.0 34.6,85.0 -100.5,7.0"
          stroke="currentColor"
          strokeWidth="1.25"
          strokeLinejoin="round"
        />
        <line
          x1="-100.5"
          y1="11.0"
          x2="34.6"
          y2="89.0"
          stroke="currentColor"
          strokeWidth="1.25"
          strokeLinecap="round"
        />
        <line
          x1="34.6"
          y1="89.0"
          x2="135.1"
          y2="31.0"
          stroke="currentColor"
          strokeWidth="1.25"
          strokeLinecap="round"
        />
      </g>
    ),
    viewBox: boxView,
  },
  build: {
    body: (
      <g>
        <polygon
          points="-60.6,35.0 0.0,70.0 0.0,44.0 -60.6,9.0"
          stroke="currentColor"
          strokeWidth="1.25"
          strokeLinejoin="round"
        />
        <polygon
          points="60.6,35.0 0.0,70.0 0.0,44.0 60.6,9.0"
          stroke="currentColor"
          strokeWidth="1.25"
          strokeLinejoin="round"
        />
        <polygon
          points="0.0,-26.0 60.6,9.0 0.0,44.0 -60.6,9.0"
          stroke="currentColor"
          strokeWidth="1.25"
          strokeLinejoin="round"
        />
        <ellipse cx="0.0" cy="-11.3" rx="2.6" ry="1.508" fill="currentColor" />
        <ellipse cx="-11.7" cy="-4.5" rx="2.6" ry="1.508" fill="currentColor" />
        <ellipse cx="-23.4" cy="2.2" rx="2.6" ry="1.508" fill="currentColor" />
        <ellipse cx="-35.1" cy="9.0" rx="2.6" ry="1.508" fill="currentColor" />
        <ellipse cx="11.7" cy="-4.5" rx="2.6" ry="1.508" fill="currentColor" />
        <ellipse cx="0.0" cy="2.2" rx="2.6" ry="1.508" fill="currentColor" />
        <ellipse cx="-11.7" cy="9.0" rx="2.6" ry="1.508" fill="currentColor" />
        <ellipse cx="-23.4" cy="15.7" rx="2.6" ry="1.508" fill="currentColor" />
        <ellipse cx="23.4" cy="2.2" rx="2.6" ry="1.508" fill="currentColor" />
        <ellipse cx="11.7" cy="9.0" rx="2.6" ry="1.508" fill="currentColor" />
        <ellipse cx="0.0" cy="15.7" rx="2.6" ry="1.508" fill="currentColor" />
        <ellipse cx="-11.7" cy="22.5" rx="2.6" ry="1.508" fill="currentColor" />
        <ellipse cx="35.1" cy="9.0" rx="2.6" ry="1.508" fill="currentColor" />
        <ellipse cx="23.4" cy="15.7" rx="2.6" ry="1.508" fill="currentColor" />
        <ellipse cx="11.7" cy="22.5" rx="2.6" ry="1.508" fill="currentColor" />
        <ellipse cx="0.0" cy="29.2" rx="2.6" ry="1.508" fill="currentColor" />
        <polygon
          points="17.3,56.0 29.4,63.0 29.4,53.0 17.3,46.0"
          stroke="currentColor"
          strokeWidth="1.25"
          strokeLinejoin="round"
        />
        <polygon
          points="55.4,48.0 29.4,63.0 29.4,53.0 55.4,38.0"
          stroke="currentColor"
          strokeWidth="1.25"
          strokeLinejoin="round"
        />
        <polygon
          points="43.3,31.0 55.4,38.0 29.4,53.0 17.3,46.0"
          stroke="currentColor"
          strokeWidth="1.25"
          strokeLinejoin="round"
        />
      </g>
    ),
    viewBox: "-66.6 -32.0 133.2 108.0",
  },
  deploy: {
    body: (
      <g>
        <polygon
          points="-60.6,35.0 34.6,90.0 34.6,46.0 -60.6,-9.0"
          stroke="currentColor"
          strokeWidth="1.25"
          strokeLinejoin="round"
        />
        <polygon
          points="95.3,55.0 34.6,90.0 34.6,46.0 95.3,11.0"
          stroke="currentColor"
          strokeWidth="1.25"
          strokeLinejoin="round"
        />
        <polygon
          points="0.0,-44.0 95.3,11.0 34.6,46.0 -60.6,-9.0"
          stroke="currentColor"
          strokeWidth="1.25"
          strokeLinejoin="round"
        />
        <line
          x1="0.0"
          y1="-36.0"
          x2="95.3"
          y2="19.0"
          stroke="currentColor"
          strokeWidth="1.25"
          strokeLinecap="round"
        />
        <line
          x1="95.3"
          y1="19.0"
          x2="34.6"
          y2="54.0"
          stroke="currentColor"
          strokeWidth="1.25"
          strokeLinecap="round"
        />
        <line
          x1="-60.6"
          y1="-1.0"
          x2="34.6"
          y2="54.0"
          stroke="currentColor"
          strokeWidth="1.25"
          strokeLinecap="round"
        />
        <ellipse
          cx="71.0"
          cy="13.0"
          rx="3"
          ry="1.7399999999999998"
          fill="currentColor"
        />
      </g>
    ),
    viewBox: "-66.6 -50.0 167.9 146.0",
  },
  dev: {
    body: (
      <g>
        <polygon
          points="-77.9,45.0 26.0,105.0 26.0,99.0 -77.9,39.0"
          stroke="currentColor"
          strokeWidth="1.25"
          strokeLinejoin="round"
        />
        <polygon
          points="103.9,60.0 26.0,105.0 26.0,99.0 103.9,54.0"
          stroke="currentColor"
          strokeWidth="1.25"
          strokeLinejoin="round"
        />
        <polygon
          points="0.0,-6.0 103.9,54.0 26.0,99.0 -77.9,39.0"
          stroke="currentColor"
          strokeWidth="1.25"
          strokeLinejoin="round"
        />
        <polygon
          points="2.6,-1.5 96.1,52.5 96.1,-17.5 2.6,-71.5"
          stroke="currentColor"
          strokeWidth="1.25"
          strokeLinejoin="round"
        />
        <polygon
          points="98.7,51.0 96.1,52.5 96.1,-17.5 98.7,-19.0"
          stroke="currentColor"
          strokeWidth="1.25"
          strokeLinejoin="round"
        />
        <polygon
          points="5.2,-73.0 98.7,-19.0 96.1,-17.5 2.6,-71.5"
          stroke="currentColor"
          strokeWidth="1.25"
          strokeLinejoin="round"
        />
        <polygon
          points="10.0,-7.8 93.1,40.2 93.1,-15.8 10.0,-63.8"
          stroke="currentColor"
          strokeWidth="1.25"
          strokeLinejoin="round"
        />
        <line
          x1="17.8"
          y1="-50.3"
          x2="42.0"
          y2="-36.3"
          stroke="currentColor"
          strokeWidth="1.25"
          strokeLinecap="round"
        />
        <line
          x1="17.8"
          y1="-42.3"
          x2="61.1"
          y2="-17.3"
          stroke="currentColor"
          strokeWidth="1.25"
          strokeLinecap="round"
        />
        <line
          x1="17.8"
          y1="-34.3"
          x2="35.1"
          y2="-24.3"
          stroke="currentColor"
          strokeWidth="1.25"
          strokeLinecap="round"
        />
        <ellipse cx="0.4" cy="13.7" rx="2.2" ry="1.276" fill="currentColor" />
        <ellipse cx="-12.6" cy="21.2" rx="2.2" ry="1.276" fill="currentColor" />
        <ellipse cx="-25.5" cy="28.7" rx="2.2" ry="1.276" fill="currentColor" />
        <ellipse cx="-38.5" cy="36.2" rx="2.2" ry="1.276" fill="currentColor" />
        <ellipse cx="10.8" cy="19.7" rx="2.2" ry="1.276" fill="currentColor" />
        <ellipse cx="-2.2" cy="27.2" rx="2.2" ry="1.276" fill="currentColor" />
        <ellipse cx="-15.2" cy="34.7" rx="2.2" ry="1.276" fill="currentColor" />
        <ellipse cx="-28.1" cy="42.2" rx="2.2" ry="1.276" fill="currentColor" />
        <ellipse cx="21.2" cy="25.7" rx="2.2" ry="1.276" fill="currentColor" />
        <ellipse cx="8.2" cy="33.2" rx="2.2" ry="1.276" fill="currentColor" />
        <ellipse cx="-4.8" cy="40.7" rx="2.2" ry="1.276" fill="currentColor" />
        <ellipse cx="-17.8" cy="48.2" rx="2.2" ry="1.276" fill="currentColor" />
        <ellipse cx="31.6" cy="31.7" rx="2.2" ry="1.276" fill="currentColor" />
        <ellipse cx="18.6" cy="39.2" rx="2.2" ry="1.276" fill="currentColor" />
        <ellipse cx="5.6" cy="46.7" rx="2.2" ry="1.276" fill="currentColor" />
        <ellipse cx="-7.4" cy="54.2" rx="2.2" ry="1.276" fill="currentColor" />
        <ellipse cx="42.0" cy="37.7" rx="2.2" ry="1.276" fill="currentColor" />
        <ellipse cx="29.0" cy="45.2" rx="2.2" ry="1.276" fill="currentColor" />
        <ellipse cx="16.0" cy="52.7" rx="2.2" ry="1.276" fill="currentColor" />
        <ellipse cx="3.0" cy="60.2" rx="2.2" ry="1.276" fill="currentColor" />
        <ellipse cx="52.4" cy="43.7" rx="2.2" ry="1.276" fill="currentColor" />
        <ellipse cx="39.4" cy="51.2" rx="2.2" ry="1.276" fill="currentColor" />
        <ellipse cx="26.4" cy="58.7" rx="2.2" ry="1.276" fill="currentColor" />
        <ellipse cx="13.4" cy="66.2" rx="2.2" ry="1.276" fill="currentColor" />
        <ellipse cx="62.8" cy="49.7" rx="2.2" ry="1.276" fill="currentColor" />
        <ellipse cx="49.8" cy="57.2" rx="2.2" ry="1.276" fill="currentColor" />
        <ellipse cx="36.8" cy="64.7" rx="2.2" ry="1.276" fill="currentColor" />
        <ellipse cx="23.8" cy="72.2" rx="2.2" ry="1.276" fill="currentColor" />
        <ellipse cx="73.2" cy="55.7" rx="2.2" ry="1.276" fill="currentColor" />
        <ellipse cx="60.2" cy="63.2" rx="2.2" ry="1.276" fill="currentColor" />
        <ellipse cx="47.2" cy="70.7" rx="2.2" ry="1.276" fill="currentColor" />
        <ellipse cx="34.2" cy="78.2" rx="2.2" ry="1.276" fill="currentColor" />
      </g>
    ),
    viewBox: "-83.9 -79.0 193.8 190.0",
  },
  env: {
    body: (
      <g>
        <polygon
          points="-41.6,0.0 -41.3,3.1 -40.2,6.2 -38.5,9.2 -36.1,12.0 -33.0,14.6 -29.4,17.0 -25.3,19.1 -20.8,20.8 -15.9,22.2 -10.8,23.2 -5.4,23.8 0.0,24.0 5.4,23.8 10.8,23.2 15.9,22.2 20.8,20.8 25.3,19.1 29.4,17.0 33.0,14.6 36.1,12.0 38.5,9.2 40.2,6.2 41.3,3.1 41.6,-0.0 41.6,-50.0 41.3,-46.9 40.2,-43.8 38.5,-40.8 36.1,-38.0 33.0,-35.4 29.4,-33.0 25.3,-30.9 20.8,-29.2 15.9,-27.8 10.8,-26.8 5.4,-26.2 0.0,-26.0 -5.4,-26.2 -10.8,-26.8 -15.9,-27.8 -20.8,-29.2 -25.3,-30.9 -29.4,-33.0 -33.0,-35.4 -36.1,-38.0 -38.5,-40.8 -40.2,-43.8 -41.3,-46.9 -41.6,-50.0"
          stroke="currentColor"
          strokeWidth="1.25"
          strokeLinejoin="round"
        />
        <polygon
          points="29.4,-33.0 25.3,-30.9 20.8,-29.2 15.9,-27.8 10.8,-26.8 5.4,-26.2 0.0,-26.0 -5.4,-26.2 -10.8,-26.8 -15.9,-27.8 -20.8,-29.2 -25.3,-30.9 -29.4,-33.0 -33.0,-35.4 -36.1,-38.0 -38.5,-40.8 -40.2,-43.8 -41.3,-46.9 -41.6,-50.0 -41.3,-53.1 -40.2,-56.2 -38.5,-59.2 -36.1,-62.0 -33.0,-64.6 -29.4,-67.0 -25.3,-69.1 -20.8,-70.8 -15.9,-72.2 -10.8,-73.2 -5.4,-73.8 -0.0,-74.0 5.4,-73.8 10.8,-73.2 15.9,-72.2 20.8,-70.8 25.3,-69.1 29.4,-67.0 33.0,-64.6 36.1,-62.0 38.5,-59.2 40.2,-56.2 41.3,-53.1 41.6,-50.0 41.3,-46.9 40.2,-43.8 38.5,-40.8 36.1,-38.0 33.0,-35.4"
          stroke="currentColor"
          strokeWidth="1.25"
        />
        <polyline
          points="-41.6,-27.5 -41.3,-24.4 -40.2,-21.3 -38.5,-18.3 -36.1,-15.5 -33.0,-12.9 -29.4,-10.5 -25.3,-8.4 -20.8,-6.7 -15.9,-5.3 -10.8,-4.3 -5.4,-3.7 0.0,-3.5 5.4,-3.7 10.8,-4.3 15.9,-5.3 20.8,-6.7 25.3,-8.4 29.4,-10.5 33.0,-12.9 36.1,-15.5 38.5,-18.3 40.2,-21.3 41.3,-24.4 41.6,-27.5"
          fill="none"
          stroke="currentColor"
          strokeWidth="1"
          strokeDasharray="3 3"
        />
      </g>
    ),
    viewBox: "-47.6 -80.0 95.2 110.0",
  },
  gomod: {
    body: (
      <g>
        <polygon
          points="-103.9,60.0 -26.0,105.0 -26.0,102.0 -103.9,57.0"
          stroke="currentColor"
          strokeWidth="1.25"
          strokeLinejoin="round"
        />
        <polygon
          points="77.9,45.0 -26.0,105.0 -26.0,102.0 77.9,42.0"
          stroke="currentColor"
          strokeWidth="1.25"
          strokeLinejoin="round"
        />
        <polygon
          points="0.0,-3.0 77.9,42.0 -26.0,102.0 -103.9,57.0"
          stroke="currentColor"
          strokeWidth="1.25"
          strokeLinejoin="round"
        />
        <line
          x1="-3.5"
          y1="10.5"
          x2="-41.6"
          y2="32.5"
          stroke="currentColor"
          strokeWidth="1.25"
          strokeLinecap="round"
        />
        <line
          x1="3.5"
          y1="14.5"
          x2="-65.8"
          y2="54.5"
          stroke="currentColor"
          strokeWidth="1.25"
          strokeLinecap="round"
        />
        <line
          x1="10.4"
          y1="18.5"
          x2="-45.0"
          y2="50.5"
          stroke="currentColor"
          strokeWidth="1.25"
          strokeLinecap="round"
        />
        <line
          x1="17.3"
          y1="22.5"
          x2="-58.9"
          y2="66.5"
          stroke="currentColor"
          strokeWidth="1.25"
          strokeLinecap="round"
        />
        <line
          x1="24.2"
          y1="26.5"
          x2="-22.5"
          y2="53.5"
          stroke="currentColor"
          strokeWidth="1.25"
          strokeLinecap="round"
        />
      </g>
    ),
    viewBox: "-109.9 -9.0 193.8 120.0",
  },
  gozo: {
    body: (
      <g>
        <polygon
          points="-52.0,30.0 0.0,60.0 0.0,56.0 -52.0,26.0"
          stroke="currentColor"
          strokeWidth="1.25"
          strokeLinejoin="round"
        />
        <polygon
          points="52.0,30.0 0.0,60.0 0.0,56.0 52.0,26.0"
          stroke="currentColor"
          strokeWidth="1.25"
          strokeLinejoin="round"
        />
        <polygon
          points="0.0,-4.0 52.0,26.0 0.0,56.0 -52.0,26.0"
          stroke="currentColor"
          strokeWidth="1.25"
          strokeLinejoin="round"
        />
        <g transform="translate(0 25.999999999999996) scale(.42) rotate(15)">
          <path
            d="M0 0 C 18 -14 20 -40 0 -44 C -20 -40 -18 -14 0 0 Z"
            transform="rotate(-90) translate(0 -3)"
            fill="currentColor"
          />
          <path
            d="M0 0 C 18 -14 20 -40 0 -44 C -20 -40 -18 -14 0 0 Z"
            transform="rotate(30) translate(0 -3)"
            fill="currentColor"
          />
          <path
            d="M0 0 C 18 -14 20 -40 0 -44 C -20 -40 -18 -14 0 0 Z"
            transform="rotate(150) translate(0 -3)"
            fill="currentColor"
          />
        </g>
      </g>
    ),
    viewBox: "-58.0 -10.0 116.0 76.0",
  },
  one: {
    body: (
      <g>
        <polygon
          points="-95.3,55.0 34.6,130.0 34.6,90.0 -95.3,15.0"
          stroke="currentColor"
          strokeWidth="1.25"
          strokeLinejoin="round"
        />
        <polygon
          points="129.9,75.0 34.6,130.0 34.6,90.0 129.9,35.0"
          stroke="currentColor"
          strokeWidth="1.25"
          strokeLinejoin="round"
        />
        <polygon
          points="0.0,-40.0 129.9,35.0 34.6,90.0 -95.3,15.0"
          stroke="currentColor"
          strokeWidth="1.25"
          strokeLinejoin="round"
        />
        <polygon
          points="-100.5,15.0 34.6,93.0 34.6,85.0 -100.5,7.0"
          stroke="currentColor"
          strokeWidth="1.25"
          strokeLinejoin="round"
        />
        <polygon
          points="135.1,35.0 34.6,93.0 34.6,85.0 135.1,27.0"
          stroke="currentColor"
          strokeWidth="1.25"
          strokeLinejoin="round"
        />
        <polygon
          points="0.0,-51.0 135.1,27.0 34.6,85.0 -100.5,7.0"
          stroke="currentColor"
          strokeWidth="1.25"
          strokeLinejoin="round"
        />
        <line
          x1="-100.5"
          y1="11.0"
          x2="34.6"
          y2="89.0"
          stroke="currentColor"
          strokeWidth="1.25"
          strokeLinecap="round"
        />
        <line
          x1="34.6"
          y1="89.0"
          x2="135.1"
          y2="31.0"
          stroke="currentColor"
          strokeWidth="1.25"
          strokeLinecap="round"
        />
        <ellipse cx="-79.5" cy="32.1" rx="1.51" ry="1.26" fill="currentColor" />
        <ellipse cx="-75.9" cy="34.2" rx="1.51" ry="1.26" fill="currentColor" />
        <ellipse cx="-72.2" cy="36.3" rx="1.51" ry="1.26" fill="currentColor" />
        <ellipse cx="-83.1" cy="34.2" rx="1.51" ry="1.26" fill="currentColor" />
        <ellipse cx="-83.1" cy="38.4" rx="1.51" ry="1.26" fill="currentColor" />
        <ellipse cx="-72.2" cy="44.7" rx="1.51" ry="1.26" fill="currentColor" />
        <ellipse cx="-68.6" cy="46.8" rx="1.51" ry="1.26" fill="currentColor" />
        <ellipse cx="-83.1" cy="42.6" rx="1.51" ry="1.26" fill="currentColor" />
        <ellipse cx="-68.6" cy="51.0" rx="1.51" ry="1.26" fill="currentColor" />
        <ellipse cx="-79.5" cy="48.9" rx="1.51" ry="1.26" fill="currentColor" />
        <ellipse cx="-75.9" cy="51.0" rx="1.51" ry="1.26" fill="currentColor" />
        <ellipse cx="-72.2" cy="53.1" rx="1.51" ry="1.26" fill="currentColor" />
        <ellipse cx="-57.7" cy="44.7" rx="1.51" ry="1.26" fill="currentColor" />
        <ellipse cx="-54.0" cy="46.8" rx="1.51" ry="1.26" fill="currentColor" />
        <ellipse cx="-50.4" cy="48.9" rx="1.51" ry="1.26" fill="currentColor" />
        <ellipse cx="-61.3" cy="46.8" rx="1.51" ry="1.26" fill="currentColor" />
        <ellipse cx="-46.8" cy="55.2" rx="1.51" ry="1.26" fill="currentColor" />
        <ellipse cx="-61.3" cy="51.0" rx="1.51" ry="1.26" fill="currentColor" />
        <ellipse cx="-46.8" cy="59.4" rx="1.51" ry="1.26" fill="currentColor" />
        <ellipse cx="-61.3" cy="55.2" rx="1.51" ry="1.26" fill="currentColor" />
        <ellipse cx="-46.8" cy="63.6" rx="1.51" ry="1.26" fill="currentColor" />
        <ellipse cx="-57.7" cy="61.5" rx="1.51" ry="1.26" fill="currentColor" />
        <ellipse cx="-54.0" cy="63.6" rx="1.51" ry="1.26" fill="currentColor" />
        <ellipse cx="-50.4" cy="65.7" rx="1.51" ry="1.26" fill="currentColor" />
        <ellipse cx="-39.5" cy="55.2" rx="1.51" ry="1.26" fill="currentColor" />
        <ellipse cx="-35.9" cy="57.3" rx="1.51" ry="1.26" fill="currentColor" />
        <ellipse cx="-32.2" cy="59.4" rx="1.51" ry="1.26" fill="currentColor" />
        <ellipse cx="-28.6" cy="61.5" rx="1.51" ry="1.26" fill="currentColor" />
        <ellipse cx="-24.9" cy="63.6" rx="1.51" ry="1.26" fill="currentColor" />
        <ellipse cx="-28.6" cy="65.7" rx="1.51" ry="1.26" fill="currentColor" />
        <ellipse cx="-32.2" cy="67.8" rx="1.51" ry="1.26" fill="currentColor" />
        <ellipse cx="-35.9" cy="69.9" rx="1.51" ry="1.26" fill="currentColor" />
        <ellipse cx="-39.5" cy="72.0" rx="1.51" ry="1.26" fill="currentColor" />
        <ellipse cx="-35.9" cy="74.1" rx="1.51" ry="1.26" fill="currentColor" />
        <ellipse cx="-32.2" cy="76.2" rx="1.51" ry="1.26" fill="currentColor" />
        <ellipse cx="-28.6" cy="78.3" rx="1.51" ry="1.26" fill="currentColor" />
        <ellipse cx="-24.9" cy="80.4" rx="1.51" ry="1.26" fill="currentColor" />
        <ellipse cx="-14.0" cy="69.9" rx="1.51" ry="1.26" fill="currentColor" />
        <ellipse cx="-10.4" cy="72.0" rx="1.51" ry="1.26" fill="currentColor" />
        <ellipse cx="-6.8" cy="74.1" rx="1.51" ry="1.26" fill="currentColor" />
        <ellipse cx="-17.7" cy="72.0" rx="1.51" ry="1.26" fill="currentColor" />
        <ellipse cx="-3.1" cy="80.4" rx="1.51" ry="1.26" fill="currentColor" />
        <ellipse cx="-17.7" cy="76.2" rx="1.51" ry="1.26" fill="currentColor" />
        <ellipse cx="-3.1" cy="84.6" rx="1.51" ry="1.26" fill="currentColor" />
        <ellipse cx="-17.7" cy="80.4" rx="1.51" ry="1.26" fill="currentColor" />
        <ellipse cx="-3.1" cy="88.8" rx="1.51" ry="1.26" fill="currentColor" />
        <ellipse cx="-14.0" cy="86.7" rx="1.51" ry="1.26" fill="currentColor" />
        <ellipse cx="-10.4" cy="88.8" rx="1.51" ry="1.26" fill="currentColor" />
        <ellipse cx="-6.8" cy="90.9" rx="1.51" ry="1.26" fill="currentColor" />
        <line
          x1="119.5"
          y1="51.0"
          x2="119.5"
          y2="69.0"
          stroke="currentColor"
          strokeWidth="1.25"
          strokeLinecap="round"
        />
        <line
          x1="115.2"
          y1="53.5"
          x2="115.2"
          y2="66.5"
          stroke="currentColor"
          strokeWidth="1.25"
          strokeLinecap="round"
        />
        <line
          x1="110.9"
          y1="56.0"
          x2="110.9"
          y2="64.0"
          stroke="currentColor"
          strokeWidth="1.25"
          strokeLinecap="round"
        />
        <line
          x1="106.5"
          y1="58.5"
          x2="106.5"
          y2="76.5"
          stroke="currentColor"
          strokeWidth="1.25"
          strokeLinecap="round"
        />
        <line
          x1="102.2"
          y1="61.0"
          x2="102.2"
          y2="74.0"
          stroke="currentColor"
          strokeWidth="1.25"
          strokeLinecap="round"
        />
        <line
          x1="97.9"
          y1="63.5"
          x2="97.9"
          y2="71.5"
          stroke="currentColor"
          strokeWidth="1.25"
          strokeLinecap="round"
        />
        <line
          x1="93.5"
          y1="66.0"
          x2="93.5"
          y2="84.0"
          stroke="currentColor"
          strokeWidth="1.25"
          strokeLinecap="round"
        />
        <line
          x1="89.2"
          y1="68.5"
          x2="89.2"
          y2="81.5"
          stroke="currentColor"
          strokeWidth="1.25"
          strokeLinecap="round"
        />
        <g transform="translate(17.3 17.0) scale(0.9) rotate(15)">
          <path
            d="M0 0 C 18 -14 20 -40 0 -44 C -20 -40 -18 -14 0 0 Z"
            transform="rotate(-90) translate(0 -3)"
            fill="currentColor"
          />
          <path
            d="M0 0 C 18 -14 20 -40 0 -44 C -20 -40 -18 -14 0 0 Z"
            transform="rotate(30) translate(0 -3)"
            fill="currentColor"
          />
          <path
            d="M0 0 C 18 -14 20 -40 0 -44 C -20 -40 -18 -14 0 0 Z"
            transform="rotate(150) translate(0 -3)"
            fill="currentColor"
          />
        </g>
      </g>
    ),
    viewBox: "-106.5 -57.0 247.6 193.0",
  },
  status: {
    body: (
      <g>
        <polygon
          points="-34.6,20.0 0.0,40.0 0.0,-0.0 -34.6,-20.0"
          stroke="currentColor"
          strokeWidth="1.25"
          strokeLinejoin="round"
        />
        <polygon
          points="34.6,20.0 0.0,40.0 0.0,-0.0 34.6,-20.0"
          stroke="currentColor"
          strokeWidth="1.25"
          strokeLinejoin="round"
        />
        <polygon
          points="0.0,-40.0 34.6,-20.0 0.0,-0.0 -34.6,-20.0"
          stroke="currentColor"
          strokeWidth="1.25"
          strokeLinejoin="round"
        />
        <polygon
          points="10.4,46.0 45.0,66.0 45.0,26.0 10.4,6.0"
          stroke="currentColor"
          strokeWidth="1.25"
          strokeLinejoin="round"
        />
        <polygon
          points="79.7,46.0 45.0,66.0 45.0,26.0 79.7,6.0"
          stroke="currentColor"
          strokeWidth="1.25"
          strokeLinejoin="round"
        />
        <polygon
          points="45.0,-14.0 79.7,6.0 45.0,26.0 10.4,6.0"
          stroke="currentColor"
          strokeWidth="1.25"
          strokeLinejoin="round"
        />
        <polygon
          points="-57.2,59.0 -22.5,79.0 -22.5,39.0 -57.2,19.0"
          stroke="currentColor"
          strokeWidth="1.25"
          strokeLinejoin="round"
        />
        <polygon
          points="12.1,59.0 -22.5,79.0 -22.5,39.0 12.1,19.0"
          stroke="currentColor"
          strokeWidth="1.25"
          strokeLinejoin="round"
        />
        <polygon
          points="-22.5,-1.0 12.1,19.0 -22.5,39.0 -57.2,19.0"
          stroke="currentColor"
          strokeWidth="1.25"
          strokeLinejoin="round"
        />
        <line
          x1="17.3"
          y1="10.0"
          x2="27.7"
          y2="16.0"
          stroke="currentColor"
          strokeWidth="1.25"
          strokeLinecap="round"
        />
        <line
          x1="5.2"
          y1="23.0"
          x2="-5.2"
          y2="29.0"
          stroke="currentColor"
          strokeWidth="1.25"
          strokeLinecap="round"
        />
      </g>
    ),
    viewBox: "-63.2 -46.0 148.9 131.0",
  },
} satisfies Record<string, { viewBox: string; body: ReactNode }>;
