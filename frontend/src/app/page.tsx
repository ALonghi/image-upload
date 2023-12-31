"use client"
import ImageUpload from '@/components/ImageUpload'
import axios from 'axios'

export default function Home() {
  const handleClick = () => {
    axios.get(`${process.env.NEXT_PUBLIC_API_URL}/`).then(res => alert(res.data)).catch(err => alert(err))
  }
  return (
    <main className="flex min-h-screen flex-col items-center justify-between p-24">
      <div className="z-10 max-w-5xl w-full items-center justify-between font-mono text-sm lg:flex">
        <p>Backend url: {process.env.NEXT_PUBLIC_API_URL}</p>
        <ImageUpload />
        <button type="button"
          onClick={() => handleClick()}
          className='disabled:cursor-not-allowed disabled:border-red-600 py-2 px-8 rounded-lg border-gray-700 border hover:bg-gray-700 hover:text-white'>
          Test call</button>
      </div>
    </main>
  )
}
